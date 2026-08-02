export type LogType = 'sent' | 'received' | 'system';

export interface LogItem {
	id: string;
	text: string;
	type: LogType;
	time: string;
}

export function getCurrentTime(): string {
	const now = new Date();
	return now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

export function createLogItem(text: string, type: LogType): LogItem {
	return {
		id: crypto.randomUUID(),
		text,
		type,
		time: getCurrentTime()
	};
}
