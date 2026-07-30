<script lang="ts">
	import { onDestroy } from 'svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		// 描画前に初期テーマを適用
		const saved = localStorage.getItem('theme') || 'dark';
		if (saved === 'system') {
			const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
			document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
		} else {
			document.documentElement.setAttribute('data-theme', saved);
		}
	});

	type Theme = 'dark' | 'light' | 'system';
	type ConnectionStatus = 'disconnected' | 'connecting' | 'waiting' | 'paired';
	type LogType = 'sent' | 'received' | 'system';

	interface LogItem {
		id: string;
		text: string;
		type: LogType;
		time: string;
	}

	let theme = $state<Theme>('dark');
	let status = $state<ConnectionStatus>('disconnected'); // 現在の通信状態
	let isPaired = $state(false); // ペアリング済みかどうか
	let socket: WebSocket | null = null; // 通信の本体
	let inputText = $state('');
	let logs = $state<LogItem[]>([]); // チャットの履歴
	let partnerText = $state<string>('');
	let lang: HTMLSelectElement | undefined = $state();
	let waiting_count = $state(0);
	let matched_count = $state(0);
	let languageNotSelected = $state(false);

	function applyTheme(targetTheme: Theme) {
		const root = document.documentElement;
		if (targetTheme === 'system') {
			const systemIsDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
			root.setAttribute('data-theme', systemIsDark ? 'dark' : 'light');
		} else {
			root.setAttribute('data-theme', targetTheme);
		}
	}

	function changeTheme(newTheme: Theme) {
		theme = newTheme;
		localStorage.setItem('theme', newTheme);
		applyTheme(newTheme);
	}

	onMount(() => {
		const savedTheme = (localStorage.getItem('theme') as Theme) || 'dark';
		theme = savedTheme;
		applyTheme(savedTheme);

		// OSのテーマ変更を監視
		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		const handleSystemChange = () => {
			if (theme === 'system') {
				applyTheme('system');
			}
		};

		mediaQuery.addEventListener('change', handleSystemChange);

		return () => mediaQuery.removeEventListener('change', handleSystemChange);
	});

	async function fetchPeopleCount() {
		try {
			const res = await fetch('http://localhost:3000/get_people_count');
			let data = await res.json();
			waiting_count = data['waiting'];
			matched_count = data['matched'];
		} catch (err) {
			console.error(err);
		}
	}

	onMount(() => {
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
		if (lang && lang.value === 'not-selected') {
			languageNotSelected = true;
			return;
		} else {
			languageNotSelected = false;
		}

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
		addLog(messageToSend, 'sent');
	}

	onDestroy(() => {
		disconnect();
	});
</script>

<main class="mx-auto max-w-6xl px-4 font-sans">
	<header class="mt-3 flex border-b border-zinc-400">
		<h1 class="m-auto mb-1 text-3xl font-bold text-base-content">Vanit</h1>
		<select
			value={theme}
			onchange={(e) => changeTheme(e.currentTarget.value as Theme)}
			class="select m-auto w-32 select-ghost"
		>
			<option value="light">☀️Lignt</option>
			<option value="dark">🌙Dark</option>
			<option value="system">💻System</option>
		</select>
	</header>

	<div>
		<ul class="mt-10 flex justify-center gap-6 text-sm text-base-content">
			<li>待機中: {waiting_count}人</li>
			<li>接続済み: {matched_count}人</li>
			<li>総ユーザー数: {waiting_count + matched_count}人</li>
		</ul>
	</div>

	<!-- ステータスバー -->
	<div
		class="mb-4 flex min-h-24 items-center justify-between rounded-2xl border border-zinc-200 bg-base-100 p-4 shadow-sm"
	>
		<div class="flex items-center gap-2.5 font-semibold">
			<span
				class="h-3 w-3 rounded-full transition-colors duration-200 {status === 'paired'
					? 'bg-green-700 shadow-[0_0_6px_rgba(46,125,50,0.5)]'
					: status === 'waiting'
						? 'bg-amber-600'
						: status === 'connecting'
							? 'bg-sky-600'
							: 'bg-red-600'}"
			></span>
			<span class="min-w-64 text-base-content">
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

		{#if languageNotSelected}
			<div role="alert" class="alert alert-warning">
				<span>Please select a language</span>
			</div>
		{/if}

		<select
			bind:this={lang}
			class="select cursor-pointer rounded-md border border-zinc-300 bg-base-100 px-3 py-1.5 text-sm text-base-content outline-none focus:border-blue-600"
		>
			<option disabled selected value="not-selected">Select your language</option>
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

		<div class="flex min-w-64 justify-end">
			{#if status === 'disconnected'}
				<button class="btn btn-outline btn-info" onclick={connect}> 接続する </button>
			{:else}
				<button class="btn btn-outline btn-secondary" onclick={disconnect}>
					{#if status === 'waiting'}
						<span class="loading loading-sm loading-dots"></span>
					{/if}
					切断する
				</button>
			{/if}
		</div>
	</div>

	<!-- やりとり表示 -->
	<div
		class="mb-4 flex flex-col gap-5 rounded-2xl border border-zinc-200 bg-base-100 p-4 shadow-sm"
	>
		<div class="flex flex-col gap-1.5">
			<span class="text-[0.85rem] font-semibold text-base-content">相手のメッセージ</span>
			<div class="flex items-center rounded-md bg-base-200 break-all text-base-content">
				{#if partnerText}
					<span
						class="min-h-[44px] w-full rounded-md border border-zinc-300 p-14 text-base transition-all duration-200"
					>
						{partnerText}
					</span>
				{:else}
					<span
						class="min-h-[44px] w-full rounded-md border border-zinc-300 p-14 text-[0.9rem] text-base-content transition-all duration-200"
					>
						（相手のメッセージがここに表示されます）
					</span>
				{/if}
			</div>
		</div>
		<div class="flex flex-col gap-1.5">
			<label for="user-input" class="text-[0.85rem] font-semibold text-base-content"
				>自分のメッセージ</label
			>
			<input
				id="user-input"
				class="min-h-[44px] w-full rounded-md border border-zinc-300 p-14 text-base transition-all duration-200 outline-none focus:enabled:border-blue-600 disabled:cursor-not-allowed disabled:bg-base-200"
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
