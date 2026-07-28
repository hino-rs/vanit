<script lang="ts">
	import { onDestroy } from 'svelte';

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
	let logsContainer: HTMLDivElement | null = $state(null);
	let partnerText = $state<string>('');
	let lang: HTMLSelectElement | undefined = $state();

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

<main class="container">
	<header class="header">
		<h2>Vanit WebSocket ペアリングチャット</h2>
		<p class="subtitle">2つのブラウザウィンドウで「接続」するとペアリングが成立します</p>
	</header>

	<!-- ステータスバー -->
	<div class="card status-card">
		<div class="status-info">
			<span class="dot {status}"></span>
			<span class="status-label">
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

		<select bind:this={lang}>
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

		<div class="actions">
			{#if status === 'disconnected'}
				<button class="btn primary" onclick={connect}>接続する</button>
			{:else}
				<button class="btn danger" onclick={disconnect}>切断する</button>
			{/if}
		</div>
	</div>

	{#if status === 'waiting'}
		<div class="banner warning">
			⏳ 相手の接続を待っています。別のタブまたはウィンドウで開いて「接続する」を押してください。
		</div>
	{/if}

	<!-- やりとり表示 -->
	<div class="card exchange-card">
		<div class="exchange-box">
			<span class="box-label">相手のメッセージ</span>
			<div class="text-display">
				{#if partnerText}
					<span class="message">{partnerText}</span>
				{:else}
					<span class="placeholder message">（相手のメッセージがここに表示されます）</span>
				{/if}
			</div>
		</div>
		<div class="exchange-box">
			<label for="user-input" class="box-label">自分のメッセージ</label>
			<input
				id="user-input"
				class="message"
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

<style>
	.container {
		max-width: 640px;
		margin: 2rem auto;
		padding: 0 1rem;
		font-family:
			system-ui,
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			Roboto,
			sans-serif;
	}

	.header {
		margin-bottom: 1.5rem;
		text-align: center;
	}

	.header h2 {
		margin: 0 0 0.25rem 0;
		color: #1a1a1a;
	}

	.subtitle {
		margin: 0;
		font-size: 0.9rem;
		color: #666;
	}

	.card {
		background: #ffffff;
		border: 1px solid #e0e0e0;
		border-radius: 10px;
		padding: 1rem;
		margin-bottom: 1rem;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.04);
	}

	.status-card {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.status-info {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-weight: 600;
	}

	.dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background-color: #9e9e9e;
	}
	.dot.paired {
		background-color: #2e7d32;
		box-shadow: 0 0 6px rgba(46, 125, 50, 0.5);
	}
	.dot.waiting {
		background-color: #ed6c02;
		animation: pulse 1.5s infinite;
	}
	.dot.connecting {
		background-color: #0288d1;
	}
	.dot.disconnected {
		background-color: #d32f2f;
	}

	@keyframes pulse {
		0% {
			opacity: 1;
		}
		50% {
			opacity: 0.4;
		}
		100% {
			opacity: 1;
		}
	}

	.banner {
		padding: 0.75rem 1rem;
		border-radius: 8px;
		font-size: 0.9rem;
		margin-bottom: 1rem;
	}
	.banner.warning {
		background-color: #fff8e1;
		border: 1px solid #ffe082;
		color: #b78103;
	}

	.exchange-card {
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
	}

	.exchange-box {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.message {
		width: 100%;
		box-sizing: border-box;
		padding: 3.5rem;
		border: 1px solid #ccc;
		border-radius: 6px;
		font-size: 1rem;
		min-height: 44px;
		transition:
			border-color 0.2s,
			box-shadow 0.2s;
	}

	.box-label {
		font-size: 0.85rem;
		font-weight: 600;
		color: #555;
	}

	.text-display {
		background-color: #f9f9f9;
		color: #1a1a1a;
		display: flex;
		align-items: center;
		word-break: break-all;
	}

	.placeholder {
		color: #9e9e9e;
		font-size: 0.9rem;
	}

	input.message {
		flex: 1;
		outline: none;
	}
	input.message:focus:not(:disabled) {
		border-color: #1976d2;
	}
	input[type='text']:disabled {
		background-color: #f5f5f5;
		cursor: not-allowed;
	}

	.btn {
		padding: 0.6rem 1.2rem;
		border: none;
		border-radius: 6px;
		font-weight: 600;
		font-size: 0.95rem;
		cursor: pointer;
		transition:
			background-color 0.2s,
			opacity 0.2s;
	}
	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.btn.primary {
		background-color: #1976d2;
		color: #ffffff;
	}
	.btn.primary:hover:not(:disabled) {
		background-color: #1565c0;
	}
	.btn.danger {
		background-color: #d32f2f;
		color: #ffffff;
	}
	.btn.danger:hover:not(:disabled) {
		background-color: #c62828;
	}
	.btn.text-btn {
		background: transparent;
		color: #666;
		padding: 0.2rem 0.5rem;
		font-size: 0.85rem;
	}
	.btn.text-btn:hover {
		color: #111;
	}

	.logs-card {
		display: flex;
		flex-direction: column;
	}

	.logs-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.logs-header h3 {
		margin: 0;
		font-size: 1rem;
		color: #424242;
	}

	.logs {
		height: 300px;
		overflow-y: auto;
		background: #fafafa;
		border: 1px solid #e0e0e0;
		border-radius: 6px;
		padding: 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.log-item {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		font-size: 0.9rem;
		line-height: 1.4;
	}

	.log-content {
		display: flex;
		gap: 0.4rem;
		word-break: break-word;
	}

	.sender-tag {
		font-weight: bold;
		user-select: none;
	}

	.time {
		font-size: 0.75rem;
		color: #888;
		margin-left: 0.5rem;
		white-space: nowrap;
	}

	.log-item.system {
		background-color: #eee;
		color: #555;
		font-style: italic;
		justify-content: center;
	}
	.log-item.system .time {
		display: none;
	}

	.log-item.sent {
		background-color: #e3f2fd;
		color: #0d47a1;
	}
	.log-item.received {
		background-color: #e8f5e9;
		color: #1b5e20;
	}

	.empty {
		color: #9e9e9e;
		text-align: center;
		margin: auto;
	}
</style>
