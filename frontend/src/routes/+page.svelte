<script lang="ts">
	import { onDestroy } from 'svelte';
	import { onMount } from 'svelte';

	type ConnectionStatus = 'disconnected' | 'connecting' | 'waiting' | 'paired';
	type LogType = 'sent' | 'received' | 'system';

	interface LogItem {
		id: string;
		text: string;
		type: LogType;
		time: string;
	}

	let status = $state<ConnectionStatus>('disconnected'); // 現在の通信状態
	let isPaired = $state(false); // ペアリング済みかどうか
	let socket: WebSocket | null = null; // 通信の本体
	let inputText = $state('');
	let logs = $state<LogItem[]>([]); // チャットの履歴
	let partnerText = $state<string>('');
	let lang: HTMLSelectElement | undefined = $state();
	let waiting_count = $state(0);
	let matched_count = $state(0);

	async function fetchPeopleCount() {
		try {
			const res = await fetch('http://localhost:3000/get_people_count');
			let data = await res.json();
			waiting_count = data["waiting"];
			matched_count = data["matched"];
		} catch (err) {
			console.error(err);
		}
	}

	onMount(() =>  {
		fetchPeopleCount();

		const interval = setInterval(fetchPeopleCount, 1000);
		
		return () => clearInterval(interval);
	});

	function getCurrentTime(): string {
		const now = new Date();
		return now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
	}

	function addLog(text: string, type: LogType) {
		logs = [...logs, { id: crypto.randomUUID(), text, type, time: getCurrentTime() }];
	}

	// $effect(() => {
	//   // ログ追加時に自動スクロール
	//   if (logs.length > 0 && logsContainer) {
	//     logsContainer.scrollTop = logsContainer.scrollHeight;
	//   }
	// });

	function connect() {
		if (socket) return;

		status = 'connecting';
		isPaired = false;
		addLog('WebSocket サーバー (ws://127.0.0.1:3000/ws) へ接続中...', 'system');

		socket = new WebSocket(`ws://127.0.0.1:3000/ws?lang=${lang ? lang.value : 'en'}`);

		socket.onopen = () => {
			status = 'waiting';
			addLog('サーバーへの接続に成功しました。ペアリング待機中...', 'system');
		};

		socket.onmessage = (event: MessageEvent<string>) => {
			const data = event.data;

			if (data === 'ペアリングが完了しました！') {
				// サーバー側で2人がマッチングしたとき
				status = 'paired';
				isPaired = true;
				addLog('ペアリングが完了しました。相手との相互通信を開始できます。', 'system');
			} else if (data === 'パートナーが切断しました。') {
				// 相手がブラウザを閉じたとき
				isPaired = false;
				status = 'disconnected';
				addLog('⚠️ パートナーが切断しました。', 'system');
				disconnect();
			} else if (data === 'パートナーへの送信に失敗しました。') {
				addLog('⚠️ パートナーへのメッセージ送信に失敗しました。', 'system');
			} else {
				// それ以外の文字列は相手からのチャットメッセージ
				addLog(data, 'received');
				partnerText = data;
			}
		};

		socket.onerror = () => {
			addLog('通信エラーが発生しました。', 'system');
		};

		socket.onclose = () => {
			status = 'disconnected';
			isPaired = false;
			addLog('WebSocket 接続が切断されました。', 'system');
			socket = null;
		};
	}

	function disconnect() {
		if (socket) {
			socket.close();
			socket = null;
		}
		status = 'disconnected';
		isPaired = false;
	}

	function sendMessage() {
		if (!isPaired || !socket) return;
		const messageToSend = inputText.trim();
		socket.send(messageToSend); // サーバーへ送信
		addLog(messageToSend, 'sent'); // 自分の画面には「自分:」として表示
		// inputText = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.isComposing) {
			sendMessage();
		}
	}

	function clearLogs() {
		logs = [];
	}

	onDestroy(() => {
		disconnect();
	});
</script>

<main class="max-w-6xl mx-auto px-4 font-sans">
	<header class="mb-6 text-center">
		<h1 class="text-3xl font-bold mb-1 text-zinc-900">Vanit</h1>

		<div>
			<ul class="flex justify-center gap-6 text-sm text-zinc-600">
				<li>待機中: {waiting_count}人</li>
				<li>接続済み: {matched_count}人</li>
				<li>総ユーザー数: {waiting_count + matched_count}人</li>
			</ul>
		</div>
	</header>

	<!-- ステータスバー -->
	<div class="bg-white border border-zinc-200 rounded-2xl p-4 mb-4 shadow-sm flex justify-between items-center">
		<div class="flex items-center gap-2.5 font-semibold">
			<span
				class="w-3 h-3 rounded-full transition-colors duration-200 {status === 'paired'
					? 'bg-green-700 shadow-[0_0_6px_rgba(46,125,50,0.5)]'
					: status === 'waiting'
						? 'bg-amber-600'
						: status === 'connecting'
							? 'bg-sky-600'
							: 'bg-red-600'}"
			></span>
			<span class="text-zinc-800">
				{#if status === 'paired'}
					ペアリング完了 (相互通信中)
				{:else if status === 'waiting'}
					ペアリング相手を待機中...
				{:else if status === 'connecting'}
					接続中...
				{:else}
					未接続
				{/if}
			</span>
		</div>

		<select bind:this={lang} class="border border-zinc-300 rounded-md px-3 py-1.5 text-sm bg-white text-zinc-800 outline-none focus:border-blue-600 cursor-pointer">
			<option value="ja">Japanese</option>
			<option value="en">English</option>
			<option value="zh">Chinese</option>
			<option value="hi">Hindi</option>
			<option value="es">Spanish</option>
			<option value="ar">Arabic</option>
			<option value="fr">French</option>
			<option value="bn">Bengali</option>
			<option value="pt">Portuguese</option>
			<option value="id">Indonesian</option>
			<option value="ur">Urdu</option>
			<option value="ru">Russian</option>
			<option value="de">German</option>
			<option value="pcm">Nigerian Pidgin</option>
			<option value="arz">Egyptian Arabic</option>
		</select>

		<div>
			{#if status === 'disconnected'}
				<button
					class="px-5 py-2.5 rounded-md font-semibold text-[0.95rem] text-white bg-blue-600 hover:enabled:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 cursor-pointer"
					onclick={connect}
				>
					接続する
				</button>
			{:else}
				<button
					class="px-5 py-2.5 rounded-md font-semibold text-[0.95rem] text-white bg-red-600 hover:enabled:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 cursor-pointer"
					onclick={disconnect}
				>
					切断する
				</button>
			{/if}
		</div>
	</div>

	{#if status === 'waiting'}
		<div class="px-4 py-3 rounded-lg text-[0.9rem] mb-4 bg-amber-50 border border-amber-300 text-amber-800">
			⏳ 相手の接続を待っています。別のタブまたはウィンドウで開いて「接続する」を押してください。
		</div>
	{/if}

	<!-- やりとり表示 -->
	<div class="bg-white border border-zinc-200 rounded-2xl p-4 mb-4 shadow-sm flex flex-col gap-5">
		<div class="flex flex-col gap-1.5">
			<span class="text-[0.85rem] font-semibold text-zinc-600">相手のメッセージ</span>
			<div class="bg-zinc-50 text-zinc-900 flex items-center break-all rounded-md">
				{#if partnerText}
					<span class="w-full p-14 border border-zinc-300 rounded-md text-base min-h-[44px] transition-all duration-200">
						{partnerText}
					</span>
				{:else}
					<span class="w-full p-14 border border-zinc-300 rounded-md text-[0.9rem] text-zinc-400 min-h-[44px] transition-all duration-200">
						（相手のメッセージがここに表示されます）
					</span>
				{/if}
			</div>
		</div>
		<div class="flex flex-col gap-1.5">
			<label for="user-input" class="text-[0.85rem] font-semibold text-zinc-600">自分のメッセージ</label>
			<input
				id="user-input"
				class="w-full p-14 border border-zinc-300 rounded-md text-base min-h-[44px] transition-all duration-200 outline-none focus:enabled:border-blue-600 disabled:bg-zinc-100 disabled:cursor-not-allowed"
				type="text"
				bind:value={inputText}
				disabled={!isPaired}
				// onchange={sendMessage}
				// onkeypress={sendMessage}
				oninput={sendMessage}
				placeholder={isPaired
					? 'メッセージを入力してください...'
					: '接続してペアリングすると入力できます'}
			/>
		</div>
	</div>
</main>
