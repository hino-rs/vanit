import { STORAGE_KEYS } from '$lib/constants/storage';

export type Theme = 'dark' | 'light' | 'system';

let themeValue = $state<Theme>('dark');

export const theme = {
	get current() {
		return themeValue;
	},
	set current(val: Theme) {
		changeTheme(val);
	}
};

export function applyTheme(targetTheme: Theme) {
	if (typeof document === 'undefined') return;
	const root = document.documentElement;
	if (targetTheme === 'system') {
		const systemIsDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
		root.setAttribute('data-theme', systemIsDark ? 'dark' : 'light');
	} else {
		root.setAttribute('data-theme', targetTheme);
	}
}

export function changeTheme(newTheme: Theme) {
	themeValue = newTheme;
	if (typeof window !== 'undefined') {
		localStorage.setItem(STORAGE_KEYS.THEME, newTheme);
	}
	applyTheme(newTheme);
}

export function initTheme() {
	if (typeof window === 'undefined') return () => {};
	const savedTheme = (localStorage.getItem(STORAGE_KEYS.THEME) as Theme) || 'dark';
	themeValue = savedTheme;
	applyTheme(savedTheme);

	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	const handleSystemChange = () => {
		if (themeValue === 'system') {
			applyTheme('system');
		}
	};

	mediaQuery.addEventListener('change', handleSystemChange);
	return () => mediaQuery.removeEventListener('change', handleSystemChange);
}
