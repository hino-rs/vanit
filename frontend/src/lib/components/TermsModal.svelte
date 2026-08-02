<script lang="ts">
	interface Props {
		dialog?: HTMLDialogElement;
	}

	let { dialog = $bindable() }: Props = $props();
</script>

<dialog bind:this={dialog} class="modal">
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
