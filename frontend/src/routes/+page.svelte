<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { CREDITS_DATA } from '$lib/data/credits';

	let creditSearch = $state('');
	let creditCategory = $state<'all' | 'frontend' | 'backend'>('all');

	let filteredCredits = $derived(
		CREDITS_DATA.filter((item) => {
			const query = creditSearch.toLowerCase().trim();
			const matchesSearch =
				!query ||
				item.name.toLowerCase().includes(query) ||
				item.description.toLowerCase().includes(query) ||
				item.license.toLowerCase().includes(query);
			const matchesCategory = creditCategory === 'all' || item.category === creditCategory;
			return matchesSearch && matchesCategory;
		})
	);

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

	let eligibilityModal: HTMLDialogElement;
	onMount(async () => {
		// 年齢制限・利用資格の確認
		const eligibility = localStorage.getItem('eligibility') || 'unconfirmed';
		if (eligibility !== 'confirmed') {
			await waitForModalClose(eligibilityModal);
		}
		localStorage.setItem('eligibility', 'confirmed');
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

	let creditModal: HTMLDialogElement;
	let privacyPolicyModal: HTMLDialogElement;
	let structureNoteModal: HTMLDialogElement;
	let termsOfServiceModal: HTMLDialogElement;
	let isFirstConnect = $state(true);
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

	function waitForModalClose(dialogElement: HTMLDialogElement): Promise<void> {
		return new Promise((resolve) => {
			dialogElement.addEventListener('close', () => resolve(), { once: true });
			dialogElement.showModal();
		});
	}

	async function connect() {
		if (socket) return;
		if (lang && lang.value === 'not-selected') {
			languageNotSelected = true;
			return;
		} else {
			languageNotSelected = false;
		}
		if (isFirstConnect) {
			await waitForModalClose(structureNoteModal);
			isFirstConnect = false;
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

<!-- 利用資格 -->
<dialog bind:this={eligibilityModal} class="modal">
	<div class="modal-box">
		<h3 class="text-lg font-bold">利用資格の確認</h3>
		<p class="py-4">
			当アプリケーションは13歳未満の利用を禁止しています。また、未成年者の利用には保護者の同意が必要です。
		</p>
		<p class="py-4">利用規約の同意も必要です。</p>
		<button class="btn" onclick={() => termsOfServiceModal.showModal()}>利用規約</button>
		<p class="py-4">
			OKボタンを押すと、利用規約と利用資格要件を満たしていることに同意したものとみなされます。要件を満たしていない場合は、このサイトを閉じてください。
		</p>
		<div class="modal-action">
			<form method="dialog">
				<button class="btn">OK</button>
			</form>
		</div>
	</div>
</dialog>

<!-- テキストがリアルタイム送信されることの警告 -->
<dialog bind:this={structureNoteModal} class="modal modal-bottom sm:modal-middle">
	<div class="modal-box">
		<h3 class="text-lg font-bold">⚠️注意</h3>
		<p class="py-4">
			送信ボタンを押す前の入力中テキストが相手に見えます。そのため、個人情報や機密情報の流出には十分気を付けてください。<strong
				>Vanitはその責任を一切負いません。</strong
			>
		</p>
		<div class="modal-action">
			<form method="dialog">
				<!-- if there is a button in form, it will close the modal -->
				<button class="btn">OK</button>
			</form>
		</div>
	</div>
</dialog>

<!-- プライバシーポリシー -->
<dialog bind:this={privacyPolicyModal} class="modal">
	<div class="modal-box">
		<h2 class="text-lg font-bold">プライバシーポリシー</h2>

		<h3 class="mt-4">チャットログの保持方針について</h3>
		<hr />
		<p>全会話内容はリアルタイム中継のみであり、サーバー上に記録・保存しません。</p>

		<h3 class="mt-4">取得するアクセス権限について</h3>
		<hr />
		<p>TODO</p>

		<h3 class="mt-4">第三者サービスの利用</h3>
		<hr />
		<p>TODO</p>

		<div class="modal-action">
			<form method="dialog">
				<button class="btn">Close</button>
			</form>
		</div>
	</div>
	<form method="dialog" class="modal-backdrop">
		<button>close</button>
	</form>
</dialog>

<!-- クレジット -->
<dialog bind:this={creditModal} class="modal">
	<div
		class="modal-box flex max-h-[85vh] min-h-[85vh] w-11/12 max-w-4xl flex-col rounded-2xl border border-base-300 bg-base-100 p-6 shadow-2xl"
	>
		<div class="flex items-center justify-between border-b border-base-300 pb-3">
			<div>
				<h2 class="flex items-center gap-2 text-2xl font-bold text-base-content">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-6 w-6 text-primary"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
						/>
					</svg>
					オープンソース・ライセンス表記
				</h2>
				<p class="mt-1 text-xs text-base-content/70">
					Vanit
					の開発には以下のオープンソースソフトウェアが使用されています。各開発者・コミュニティに感謝いたします。
				</p>
			</div>
		</div>

		<!-- フィルター・検索バー -->
		<div class="flex flex-wrap items-center justify-between gap-3 py-4">
			<!-- カテゴリタブ -->
			<div class="join rounded-xl bg-base-200 p-1">
				<button
					class="btn join-item border-none btn-xs sm:btn-sm {creditCategory === 'all'
						? 'shadow-sm btn-primary'
						: 'btn-ghost text-base-content/70'}"
					onclick={() => (creditCategory = 'all')}
				>
					すべて ({CREDITS_DATA.length})
				</button>
				<button
					class="btn join-item border-none btn-xs sm:btn-sm {creditCategory === 'frontend'
						? 'shadow-sm btn-primary'
						: 'btn-ghost text-base-content/70'}"
					onclick={() => (creditCategory = 'frontend')}
				>
					Frontend ({CREDITS_DATA.filter((c) => c.category === 'frontend').length})
				</button>
				<button
					class="btn join-item border-none btn-xs sm:btn-sm {creditCategory === 'backend'
						? 'shadow-sm btn-primary'
						: 'btn-ghost text-base-content/70'}"
					onclick={() => (creditCategory = 'backend')}
				>
					Backend ({CREDITS_DATA.filter((c) => c.category === 'backend').length})
				</button>
			</div>

			<div class="flex max-w-xs flex-1 gap-2">
				<!-- 検索インプット -->
				<input
					type="text"
					bind:value={creditSearch}
					placeholder="ライブラリ名・ライセンスで検索..."
					class="input-bordered input w-full rounded-lg input-sm"
				/>
			</div>
		</div>

		<!-- リスト表示エリア -->
		<div class="my-2 flex-1 space-y-3 overflow-y-auto pr-1">
			{#if filteredCredits.length === 0}
				<div class="py-10 text-center text-base-content/60">
					一致するライブラリが見つかりませんでした。
				</div>
			{:else}
				{#each filteredCredits as item (item.name)}
					<div
						class="rounded-xl border border-base-300 bg-base-200/50 p-4 transition-all duration-200 hover:border-primary/40"
					>
						<div class="flex items-start justify-between gap-2">
							<div class="flex flex-wrap items-center gap-2">
								<a
									href={item.homepage}
									target="_blank"
									rel="noopener noreferrer"
									class="flex items-center gap-1 text-base font-bold text-primary hover:underline"
								>
									{item.name}
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-3.5 w-3.5 opacity-70"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
										/>
									</svg>
								</a>
								<span class="badge badge-outline font-mono text-xs badge-sm">{item.version}</span>
								<span class="badge text-xs badge-sm capitalize badge-neutral">{item.category}</span>
							</div>
							<span class="badge text-xs badge-sm font-semibold whitespace-nowrap badge-accent">
								{item.license}
							</span>
						</div>

						<p class="mt-2 text-xs leading-relaxed text-base-content/80">
							{item.description}
						</p>

						{#if item.author}
							<div class="mt-2 flex items-center gap-1 text-[0.75rem] text-base-content/60">
								<span>© {item.author}</span>
							</div>
						{/if}
					</div>
				{/each}
			{/if}
		</div>

		<!-- フッターアクション -->
		<div class="modal-action mt-2 flex items-center justify-between border-t border-base-300 pt-3">
			<span class="text-xs text-base-content/60">
				表示中: {filteredCredits.length} / {CREDITS_DATA.length} 件
			</span>
			<form method="dialog">
				<button class="btn btn-ghost btn-sm">閉じる</button>
			</form>
		</div>
	</div>
	<form method="dialog" class="modal-backdrop">
		<button>close</button>
	</form>
</dialog>


<!-- 利用規約 -->
<dialog bind:this={termsOfServiceModal} class="modal">
	<div class="modal-box w-11/12 max-w-5xl">
		<h2 class="text-2xl font-extrabold">利用規約</h2>

		<section class="mt-4 mb-4">
			<h3 class="bm-2 text-xl font-bold">禁止事項</h3>
			<ul>
				<li>
					<strong>ハラスメント・誹謗中傷:</strong> 相手を不快にさせる発言、ヘイトスピーチ、性的な表現、脅迫など。
				</li>
				<li>
					<strong>晒し行為・プライバシー侵害:</strong>
					チャット画面のスクリーンショットや録画を、相手に許可なくSNSに公開・拡散する行為。
				</li>
				<li>
					<strong>違法行為・スパム:</strong> 詐欺、外部サイトへの悪質な誘導・勧誘・bot等による自動接続。
				</li>
			</ul>
		</section>

		<section class="mb-4">
			<h3 class="bm-2 text-xl font-bold">免責事項</h3>
			<p>以下の項目について運営は一切の責任を負いません。</p>
			<ul>
				<li>
					<strong>ユーザー間トラブル:</strong> ユーザー同士の会話内容や発生したトラブル（損害・精神的苦痛等）
				</li>
				<li>
					<strong>意図しない情報漏洩:</strong> ユーザーが誤って個人情報を入力・送信したことによる損害
				</li>
				<li>
					<strong>サービスの中断・終了:</strong> サーバーダウン、メンテナンス、または予約の無いサービス内容変更・終了
				</li>
			</ul>
		</section>

		<section class="mb-4">
			<h3 class="bm-2 text-xl font-bold">違反者への対応</h3>
			<p>運営は利用規約に違反したユーザーに対し、事前予告なくアクセス遮断を行う権限を有します。</p>
		</section>

		<section class="mb-4">
			<h3 class="bm-2 text-xl font-bold">利用資格</h3>
			<ul>
				<li>13歳以上であり、未成年者の場合は保護者の同意があること</li>
				<li>利用規約に同意していること</li>
			</ul>
		</section>

		<p class="py-4"></p>
		<div class="modal-action">
			<form method="dialog">
				<button class="btn">Close</button>
			</form>
		</div>
	</div>
</dialog>

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

<main class="mx-auto max-w-6xl px-4 font-sans">
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
						class="min-h-11 w-full rounded-md border border-zinc-300 p-14 text-base transition-all duration-200"
					>
						{partnerText}
					</span>
				{:else}
					<span
						class="min-h-11 w-full rounded-md border border-zinc-300 p-14 text-[0.9rem] text-base-content transition-all duration-200"
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
				class="min-h-11 w-full rounded-md border border-zinc-300 p-14 text-base transition-all duration-200 outline-none focus:enabled:border-blue-600 disabled:cursor-not-allowed disabled:bg-base-200"
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

<footer class="mt-3 items-center justify-center text-center">
	<div class="mt-3 flex justify-center gap-32">
		<button class="btn" onclick={() => termsOfServiceModal.showModal()}>利用規約</button>
		<button class="btn" onclick={() => privacyPolicyModal.showModal()}>プライバシーポリシー</button>
		<button class="btn" onclick={() => creditModal.showModal()}>クレジット</button>
	</div>
	<p class="mt-4">© 2026 Vanit by hino-rs. All Rights Reserved.</p>
</footer>
