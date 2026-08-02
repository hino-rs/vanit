import type * as Types from '$lib/types';

export const API_BASE_URL = 'http://localhost:3000';

export async function fetchPeopleCount(): Promise<{ waiting: number; matched: number }> {
	try {
		const res = await fetch(`${API_BASE_URL}/api/get_people_count`);
		if (!res.ok) throw new Error('Failed to fetch people count');
		const data = await res.json();
		return {
			waiting: data['waiting'] ?? 0,
			matched: data['matched'] ?? 0
		};
	} catch (err) {
		console.error(err);
		return { waiting: 0, matched: 0 };
	}
}

export async function reportPartner(id: string, reason: Types.ReportReason, chat: string[]) {
	const reportRequest: Types.ReportRequest = {
		target_user_id: id,
		reason: reason,
		chat: chat
	};
	try {
		await fetch(`${API_BASE_URL}/api/report`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(reportRequest)
		});
	} catch (err) {
		console.error(err);
	}
}
