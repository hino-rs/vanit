export interface CreditItem {
	name: string;
	version: string;
	license: string;
	category: 'frontend' | 'backend';
	isDirect: boolean;
	description: string;
	homepage: string;
	author?: string;
}

export const CREDITS_DATA: CreditItem[] = [
	// Frontend Direct Dependencies
	{
		name: 'svelte',
		version: '5.56.1',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'Cybernetically enhanced web apps framework with reactive primitives.',
		homepage: 'https://svelte.dev',
		author: 'Rich Harris & Svelte contributors'
	},
	{
		name: '@sveltejs/kit',
		version: '2.63.0',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'The official application framework for Svelte.',
		homepage: 'https://kit.svelte.dev',
		author: 'Svelte team'
	},
	{
		name: 'vite',
		version: '8.0.16',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'Next generation frontend tooling and lightning-fast dev server.',
		homepage: 'https://vitejs.dev',
		author: 'Evan You & Vite contributors'
	},
	{
		name: 'tailwindcss',
		version: '4.3.0',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'A utility-first CSS framework for rapid UI development.',
		homepage: 'https://tailwindcss.com',
		author: 'Adam Wathan & Tailwind Labs'
	},
	{
		name: 'daisyui',
		version: '5.7.4',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'The most popular component library for Tailwind CSS.',
		homepage: 'https://daisyui.com',
		author: 'Pouya Saadeghi'
	},
	{
		name: 'typescript',
		version: '6.0.3',
		license: 'Apache-2.0',
		category: 'frontend',
		isDirect: true,
		description: 'Typed superset of JavaScript that compiles to plain JavaScript.',
		homepage: 'https://www.typescriptlang.org',
		author: 'Microsoft Corp.'
	},
	{
		name: '@sveltejs/vite-plugin-svelte',
		version: '7.1.2',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'Official Svelte plugin for Vite.',
		homepage: 'https://github.com/sveltejs/vite-plugin-svelte',
		author: 'Svelte team'
	},
	{
		name: '@tailwindcss/vite',
		version: '4.3.0',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'Vite plugin integration for Tailwind CSS v4.',
		homepage: 'https://tailwindcss.com',
		author: 'Tailwind Labs'
	},
	{
		name: 'eslint',
		version: '10.4.1',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'An extensible JavaScript linter for code quality and patterns.',
		homepage: 'https://eslint.org',
		author: 'JS Open Source Foundation & contributors'
	},
	{
		name: 'prettier',
		version: '3.8.3',
		license: 'MIT',
		category: 'frontend',
		isDirect: true,
		description: 'An opinionated code formatter supporting Svelte, TS, HTML, and CSS.',
		homepage: 'https://prettier.io',
		author: 'James Long & Prettier contributors'
	},
	{
		name: 'prettier-plugin-svelte',
		version: '4.1.0',
		license: 'MIT',
		category: 'frontend',
		isDirect: false,
		description: 'Prettier plugin for formatting Svelte template files.',
		homepage: 'https://github.com/sveltejs/prettier-plugin-svelte',
		author: 'Svelte team'
	},
	{
		name: 'prettier-plugin-tailwindcss',
		version: '0.8.0',
		license: 'MIT',
		category: 'frontend',
		isDirect: false,
		description: 'Prettier plugin for automatic class sorting in Tailwind CSS.',
		homepage: 'https://github.com/tailwindlabs/prettier-plugin-tailwindcss',
		author: 'Tailwind Labs'
	},

	// Backend Direct & Key Dependencies
	{
		name: 'axum',
		version: '0.8.9',
		license: 'MIT',
		category: 'backend',
		isDirect: true,
		description: 'Ergonomic and modular web framework built with Tokio, Tower, and Hyper.',
		homepage: 'https://github.com/tokio-rs/axum',
		author: 'Tokio Contributors'
	},
	{
		name: 'tokio',
		version: '1.53.1',
		license: 'MIT',
		category: 'backend',
		isDirect: true,
		description: 'An event-driven, non-blocking I/O platform for writing asynchronous applications.',
		homepage: 'https://tokio.rs',
		author: 'Tokio Contributors'
	},
	{
		name: 'tower-http',
		version: '0.7.0',
		license: 'MIT',
		category: 'backend',
		isDirect: true,
		description: 'HTTP specific tower middleware and utilities including CORS support.',
		homepage: 'https://github.com/tower-rs/tower-http',
		author: 'Tower Contributors'
	},
	{
		name: 'dashmap',
		version: '6.2.1',
		license: 'MIT',
		category: 'backend',
		isDirect: true,
		description: 'Blazingly fast concurrent hash map implementation in Rust.',
		homepage: 'https://github.com/xacrimon/dashmap',
		author: 'Acrimon'
	},
	{
		name: 'serde',
		version: '1.0.229',
		license: 'MIT / Apache-2.0',
		category: 'backend',
		isDirect: true,
		description: 'Generic serialization/deserialization framework for Rust data structures.',
		homepage: 'https://serde.rs',
		author: 'Erick Tryzelaar & David Tolnay'
	},
	{
		name: 'serde_json',
		version: '1.0.151',
		license: 'MIT / Apache-2.0',
		category: 'backend',
		isDirect: true,
		description: 'Strongly typed JSON serialization and deserialization library.',
		homepage: 'https://github.com/serde-rs/json',
		author: 'Erick Tryzelaar & David Tolnay'
	},
	{
		name: 'anyhow',
		version: '1.0.104',
		license: 'MIT / Apache-2.0',
		category: 'backend',
		isDirect: true,
		description: 'Flexible concrete Error type for easy idiomatic error handling in Rust.',
		homepage: 'https://github.com/dtolnay/anyhow',
		author: 'David Tolnay'
	},
	{
		name: 'futures',
		version: '0.3.33',
		license: 'MIT / Apache-2.0',
		category: 'backend',
		isDirect: true,
		description: 'Zero-cost asynchronous programming primitives for Rust.',
		homepage: 'https://github.com/rust-lang/futures-rs',
		author: 'Rust Project Developers'
	},
	{
		name: 'uuid',
		version: '1.24.0',
		license: 'Apache-2.0 / MIT',
		category: 'backend',
		isDirect: true,
		description: 'Library to generate and parse Universally Unique Identifiers (UUIDs).',
		homepage: 'https://github.com/uuid-rs/uuid',
		author: 'Ashley Mannix & Rust Developers'
	},
	{
		name: 'log',
		version: '0.4.33',
		license: 'MIT / Apache-2.0',
		category: 'backend',
		isDirect: true,
		description: 'Lightweight logging facade for Rust libraries and applications.',
		homepage: 'https://github.com/rust-lang/log',
		author: 'Rust Project Developers'
	},
	{
		name: 'env_logger',
		version: '0.11.11',
		license: 'MIT / Apache-2.0',
		category: 'backend',
		isDirect: true,
		description: 'Logging implementation for log facade configurable via environment variables.',
		homepage: 'https://github.com/rust-cli/env_logger',
		author: 'The Rust CLI Developers'
	},
	{
		name: 'hyper',
		version: '1.5.0',
		license: 'MIT',
		category: 'backend',
		isDirect: false,
		description: 'Fast and correct HTTP implementation for Rust.',
		homepage: 'https://hyper.rs',
		author: 'Sean McArthur'
	},
	{
		name: 'tower',
		version: '0.5.0',
		license: 'MIT',
		category: 'backend',
		isDirect: false,
		description: 'Modular component framework for async networking clients and servers.',
		homepage: 'https://github.com/tower-rs/tower',
		author: 'Tower Contributors'
	},
];
