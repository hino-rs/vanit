<script lang="ts">
	import type { SvelteMap } from 'svelte/reactivity';
	import * as Types from '$lib/types';
	import { reportPartner } from '$lib/api/client';

	interface Props {
		dialog?: HTMLDialogElement;
		partnersRecord: SvelteMap<string, string[]>;
	}

	let { dialog = $bindable(), partnersRecord }: Props = $props();
</script>

<dialog bind:this={dialog} class="modal">
	<div class="modal-box h-128">
		<form method="dialog">
			<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm">✕</button>
		</form>
		<h3 class="text-lg font-bold">パートナー履歴</h3>
		{#if partnersRecord.size === 0}
			<p class="py-4 text-sm text-base-content/60">まだ履歴はありません。</p>
		{:else}
			<ul class="list mt-4 space-y-3">
				{#each partnersRecord as [partner_id, chat], i (partner_id)}
					<li
						class="list-row flex items-center justify-between gap-2 rounded-lg border border-base-300 p-3"
					>
						<div class="flex-1 overflow-hidden">
							<div class="font-mono text-xs text-base-content/70">ID: {partner_id}</div>
							<div class="mt-1 text-sm font-medium text-base-content">
								{#if chat.length > 0}
									{chat.join(' / ')}
								{:else}
									<span class="text-base-content/50 italic">（メッセージなし）</span>
								{/if}
							</div>
						</div>
						<div class="relative">
							<button
								class="btn btn-sm btn-warning"
								popovertarget="popover-{i}"
								style="anchor-name:--anchor-{i}"
							>
								通報する
							</button>
							<ul
								class="menu dropdown w-52 rounded-box border border-base-300 bg-base-100 p-2 shadow-lg"
								popover
								id="popover-{i}"
								style="position-anchor:--anchor-{i}"
							>
								{#each Types.REPORT_REASON as reason}
									<li>
										<button
											class="text-xs"
											onclick={async () => reportPartner(partner_id, reason, chat)}
										>
											{reason}
										</button>
									</li>
								{/each}
							</ul>
						</div>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
	<form method="dialog" class="modal-backdrop">
		<button>close</button>
	</form>
</dialog>
