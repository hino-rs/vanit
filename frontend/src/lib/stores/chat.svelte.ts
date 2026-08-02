import { SvelteMap } from 'svelte/reactivity';
import { STORAGE_KEYS } from '$lib/constants/storage';
import * as Types from '$lib/types';
import { getOrCreateDeviceId } from '$lib/utils/device';
import { createLogItem, type LogItem, type LogType } from '$lib/utils/log';

export type ConnectionStatus = 'disconnected' | 'connecting' | 'waiting' | 'paired';

class ChatStore {
	socket = $state<WebSocket | null>(null);
	status = $state<ConnectionStatus>('disconnected');
	isPaired = $state(false);
	partnersRecord = new SvelteMap<string, string[]>();
	inputText = $state('');
	partnerText = $state('');
	logs = $state<LogItem[]>([]);
	userId = $state('');
	partnerId = $state('');
	isFirstConnect = $state(true);
	languageNotSelected = $state(false);
	lastMessageTime: number | null = null;
	elapsedSeconds: number | null = null;

	init() {
		if (typeof window !== 'undefined') {
			this.userId = getOrCreateDeviceId();
			const hasConnected = localStorage.getItem(STORAGE_KEYS.HAS_CONNECTED);
			if (hasConnected === 'true') {
				this.isFirstConnect = false;
			}
		}
	}

	addLog(text: string, type: LogType) {
		this.logs = [...this.logs, createLogItem(text, type)];
	}

	async connect(selectedLang?: string, onFirstConnectModal?: () => Promise<void>) {
		if (this.socket) return;

		if (!selectedLang || selectedLang === 'not-selected') {
			this.languageNotSelected = true;
			return;
		} else {
			this.languageNotSelected = false;
		}

		if (this.isFirstConnect) {
			if (onFirstConnectModal) {
				await onFirstConnectModal();
			}
			this.isFirstConnect = false;
			if (typeof localStorage !== 'undefined') {
				localStorage.setItem(STORAGE_KEYS.HAS_CONNECTED, 'true');
			}
		}

		this.status = 'connecting';
		this.isPaired = false;
		this.addLog('WebSocket サーバー (ws://127.0.0.1:3000/ws) へ接続中...', 'system');

		const wsUrl = `ws://127.0.0.1:3000/ws?user_id=${this.userId}&lang=${selectedLang}`;

		this.socket = new WebSocket(wsUrl);

		this.socket.onopen = () => {
			this.status = 'waiting';
			this.addLog('サーバーへの接続に成功しました。ペアリング待機中...', 'system');
		};

		this.socket.onmessage = (event: MessageEvent) => {
			try {
				const data: Types.Message = JSON.parse(event.data);
				if (data.type === 'system') {
					if (data.event.type === 'matching_completed') {
						this.status = 'paired';
						this.isPaired = true;
						this.partnerId = data.event.partner_id;
						if (!this.partnersRecord.has(this.partnerId)) {
							this.partnersRecord.set(this.partnerId, []);
						}
						this.lastMessageTime = null;
					} else if (data.event.type === 'partner_disconnected') {
						this.disconnect();
						this.lastMessageTime = null;
						this.elapsedSeconds = null;
						console.log(this.partnersRecord.values());
					} else if (data.event.type === 'failed_to_send_message') {
						console.error('メッセージの送信に失敗');
					}
				} else if (data.type === 'chat') {
					const currentTime = performance.now();
					this.partnerText = data.content;

					let currentRecord = this.partnersRecord.get(this.partnerId) ?? [];
					const isNewMessageBlock =
						this.lastMessageTime === null ||
						(currentTime - this.lastMessageTime) / 1000 > 1.0 ||
						currentRecord.length === 0;

					if (isNewMessageBlock) {
						currentRecord = [...currentRecord, this.partnerText];
					} else {
						currentRecord = [...currentRecord.slice(0, -1), this.partnerText];
					}
					this.partnersRecord.set(this.partnerId, currentRecord);
					this.lastMessageTime = currentTime;
				} else {
					console.error('不明なメッセージ');
				}
			} catch (err) {
				console.error('メッセージのパースに失敗:', err);
			}
		};

		this.socket.onerror = () => {
			this.addLog('通信エラーが発生しました。', 'system');
		};

		this.socket.onclose = () => {
			this.status = 'disconnected';
			this.isPaired = false;
			this.addLog('WebSocket 接続が切断されました。', 'system');
			this.socket = null;
			this.partnerText = '';
		};
	}

	disconnect() {
		if (this.socket) {
			this.socket.close();
		}
	}

	sendMessage() {
		if (!this.isPaired || !this.socket) return;
		const messageToSend = this.inputText.trim();
		let data: Types.Message = {
			type: 'chat',
			content: messageToSend
		};
		this.socket.send(JSON.stringify(data));
		this.addLog(messageToSend, 'sent');
	}
}

export const chatStore = new ChatStore();
export const chat = chatStore;
