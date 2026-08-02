<script lang="ts">
	interface Props {
		dialog?: HTMLDialogElement;
		onOpenTerms?: () => void;
	}

	let { dialog = $bindable(), onOpenTerms }: Props = $props();
</script>

<dialog bind:this={dialog} class="modal">
	<div class="modal-box">
		<h3 class="text-lg font-bold">利用資格の確認</h3>
		<p class="py-4">
			当アプリケーションは13歳未満の利用を禁止しています。また、未成年者の利用には保護者の同意が必要です。
		</p>
		<p class="py-4">利用規約の同意も必要です。</p>
		<button class="btn" onclick={() => onOpenTerms?.()}>利用規約</button>
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
