<script lang="ts">
	interface Props {
		dialog?: HTMLDialogElement;
	}

	let { dialog = $bindable() }: Props = $props();
</script>

<dialog bind:this={dialog} class="modal">
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
