<script lang="ts">
  import { onDestroy } from 'svelte';

  // ステータス管理
  let status = $state<'disconnected' | 'connecting' | 'connected'>('disconnected');
  let socket: WebSocket | null = null;
  let inputText = $state('');
  let logs = $state<Array<{ id: string; text: string; type: 'sent' | 'received' | 'system' }>>([]);

  // ログの追加
  function addLog(text: string, type: 'sent' | 'received' | 'system') {
    logs = [...logs, { id: crypto.randomUUID(), text, type }];
  }

  // WebSocket 接続
  function connect() {
    if (socket) return;

    status = 'connecting';
    addLog('ws://127.0.0.1:3000/ws へ接続中...', 'system');

    socket = new WebSocket('ws://127.0.0.1:3000/ws');

    socket.onopen = () => {
      status = 'connected';
      addLog('接続に成功しました！', 'system');
    };

    socket.onmessage = (event) => {
      addLog(`[受信] ${event.data}`, 'received');
    };

    socket.onerror = () => {
      addLog('エラーが発生しました', 'system');
    };

    socket.onclose = () => {
      status = 'disconnected';
      addLog('切断されました', 'system');
      socket = null;
    };
  }

  // WebSocket 切断
  function disconnect() {
    if (socket) {
      socket.close();
    }
  }

  // メッセージ送信
  function sendMessage() {
    if (!inputText.trim() || status !== 'connected' || !socket) return;

    socket.send(inputText);
    addLog(`[送信] ${inputText}`, 'sent');
    inputText = '';
  }

  // Enterキーで送信
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      sendMessage();
    }
  }

  // コンポーネント破棄時にソケットをクローズ
  onDestroy(() => {
    disconnect();
  });
</script>

<main class="container">
  <h2>Axum WebSocket テスター</h2>

  <!-- 接続状態 & コントロール -->
  <div class="card status-bar">
    <div class="status-indicator">
      <span class="dot {status}"></span>
      <span class="status-text">
        {#if status === 'connected'} 接続済み
        {:else if status === 'connecting'} 接続中...
        {:else} 切断状態
        {/if}
      </span>
    </div>

    <div>
      {#if status === 'disconnected'}
        <button class="btn primary" onclick={connect}>接続する</button>
      {:else}
        <button class="btn danger" onclick={disconnect} disabled={status === 'connecting'}>切断する</button>
      {/if}
    </div>
  </div>

  <!-- メッセージ入力フォーム -->
  <div class="card send-form">
    <input
      type="text"
      placeholder={status === 'connected' ? 'メッセージを入力...' : '接続してください'}
      bind:value={inputText}
      onkeydown={handleKeydown}
      disabled={status !== 'connected'}
    />
    <button class="btn primary" onclick={sendMessage} disabled={status !== 'connected' || !inputText.trim()}>
      送信
    </button>
  </div>

  <!-- ログ表示領域 -->
  <div class="card logs-container">
    <h3>通信ログ</h3>
    <div class="logs">
      {#each logs as log (log.id)}
        <div class="log-item {log.type}">
          {log.text}
        </div>
      {:else}
        <div class="empty">ログはまだありません</div>
      {/each}
    </div>
  </div>
</main>

<style>
  .container {
    max-width: 600px;
    margin: 2rem auto;
    padding: 0 1rem;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .card {
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: bold;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background-color: #6c757d;
  }
  .dot.connected { background-color: #28a745; }
  .dot.connecting { background-color: #ffc107; }
  .dot.disconnected { background-color: #dc3545; }

  .send-form {
    display: flex;
    gap: 0.5rem;
  }

  input[type="text"] {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 1rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    font-weight: bold;
    cursor: pointer;
  }
  .btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn.primary { background-color: #0d6efd; color: white; }
  .btn.danger { background-color: #dc3545; color: white; }

  .logs-container h3 {
    margin-top: 0;
    margin-bottom: 0.5rem;
    font-size: 1rem;
    color: #495057;
  }

  .logs {
    height: 250px;
    overflow-y: auto;
    background: #ffffff;
    border: 1px solid #ced4da;
    border-radius: 4px;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .log-item {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9rem;
  }
  .log-item.system { color: #6c757d; font-style: italic; }
  .log-item.sent { background-color: #e7f5ff; color: #1864ab; }
  .log-item.received { background-color: #ebfbee; color: #2b8a3e; }

  .empty {
    color: #adb5bd;
    text-align: center;
    margin-top: 2rem;
  }
</style>
