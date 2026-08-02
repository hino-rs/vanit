<script lang="ts">
	import { CREDITS_DATA } from '$lib/data/credits';

	interface Props {
		dialog?: HTMLDialogElement;
	}

	let { dialog = $bindable() }: Props = $props();

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
</script>

<dialog bind:this={dialog} class="modal">
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
