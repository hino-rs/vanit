<script lang="ts">
	interface Props {
		dialog?: HTMLDialogElement;
	}

	let { dialog = $bindable() }: Props = $props();
</script>

<dialog bind:this={dialog} class="modal modal-bottom sm:modal-middle">
	<div class="modal-box">
		<h3 class="text-lg font-bold">⚠️注意</h3>
		<p class="py-4">
			送信ボタンを押す前の入力中テキストが相手に見えます。そのため、個人情報や機密情報の流出には十分気を付けてください。<strong
				>Vanitはその責任を一切負いません。</strong
			>
		</p>
		<div class="modal-action">
			<form method="dialog">
				<button class="btn">OK</button>
			</form>
		</div>
	</div>
</dialog>
