import { STORAGE_KEYS } from '$lib/constants/storage';

export function getOrCreateDeviceId(): string {
	let id = localStorage.getItem(STORAGE_KEYS.DEVICE_ID);
	if (!id) {
		id = crypto.randomUUID();
		localStorage.setItem(STORAGE_KEYS.DEVICE_ID, id);
	}
	return id;
}
