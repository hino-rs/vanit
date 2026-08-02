<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { STORAGE_KEYS } from '$lib/constants/storage';
	import { fetchPeopleCount } from '$lib/api/client';
	import { initTheme, theme } from '$lib/stores/theme.svelte';
	import { chatStore } from '$lib/stores/chat.svelte';

	import EligibilityModal from '$lib/components/EligibilityModal.svelte';
	import StructureNoteModal from '$lib/components/StructureNoteModal.svelte';
	import PrivacyPolicyModal from '$lib/components/PrivacyPolicyModal.svelte';
	import TermsModal from '$lib/components/TermsModal.svelte';
	import PartnersRecordModal from '$lib/components/PartnersRecordModal.svelte';
	import CreditsList from '$lib/components/CreditsList.svelte';

	let eligibilityModal: HTMLDialogElement | undefined = $state();
	let creditModal: HTMLDialogElement | undefined = $state();
	let privacyPolicyModal: HTMLDialogElement | undefined = $state();
	let structureNoteModal: HTMLDialogElement | undefined = $state();
	let termsOfServiceModal: HTMLDialogElement | undefined = $state();
	let partnersRecordModal: HTMLDialogElement | undefined = $state();

	let lang: HTMLSelectElement | undefined = $state();
	let waiting_count = $state(0);
	let matched_count = $state(0);

	function waitForModalClose(dialogElement: HTMLDialogElement): Promise<void> {
		return new Promise((resolve) => {
			dialogElement.addEventListener('close', () => resolve(), { once: true });
			dialogElement.showModal();
		});
	}

	onMount(() => {
		const cleanupTheme = initTheme();
		chatStore.init();

		// 年齢制限・利用資格の確認
		const checkEligibility = async () => {
			const eligibility = localStorage.getItem(STORAGE_KEYS.ELIGIBILITY) || 'unconfirmed';
			if (eligibility !== 'confirmed' && eligibilityModal) {
				await waitForModalClose(eligibilityModal);
			}
			localStorage.setItem(STORAGE_KEYS.ELIGIBILITY, 'confirmed');
		};
		checkEligibility();

		// 人数取得のポーリング
		const updatePeopleCount = async () => {
			const res = await fetchPeopleCount();
			waiting_count = res.waiting;
			matched_count = res.matched;
		};
		updatePeopleCount();
		const interval = setInterval(updatePeopleCount, 1000);

		return () => {
			cleanupTheme();
			clearInterval(interval);
		};
	});

	async function handleConnect() {
		await chatStore.connect(lang?.value, async () => {
			if (structureNoteModal) {
				await waitForModalClose(structureNoteModal);
			}
		});
	}

	onDestroy(() => {
		chatStore.disconnect();
	});
</script>

<EligibilityModal
	bind:dialog={eligibilityModal}
	onOpenTerms={() => termsOfServiceModal?.showModal()}
/>
<StructureNoteModal bind:dialog={structureNoteModal} />
<PrivacyPolicyModal bind:dialog={privacyPolicyModal} />
<TermsModal bind:dialog={termsOfServiceModal} />
<PartnersRecordModal bind:dialog={partnersRecordModal} partnersRecord={chatStore.partnersRecord} />
<CreditsList bind:dialog={creditModal} />

<header class="mt-3 flex border-b border-zinc-400">
	<h1 class="m-auto mb-1 text-3xl font-bold text-base-content">Vanit</h1>
	<select
		bind:value={theme.current}
		class="select m-auto w-32 select-ghost"
	>
		<option value="light">☀️Lignt</option>
		<option value="dark">🌙Dark</option>
		<option value="system">💻System</option>
	</select>
	<button class="btn m-auto" onclick={() => partnersRecordModal?.showModal()}>履歴</button>
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
				class="h-3 w-3 rounded-full transition-colors duration-200 {chatStore.status === 'paired'
					? 'bg-green-700 shadow-[0_0_6px_rgba(46,125,50,0.5)]'
					: chatStore.status === 'waiting'
						? 'bg-amber-600'
						: chatStore.status === 'connecting'
							? 'bg-sky-600'
							: 'bg-red-600'}"
			></span>
			<span class="min-w-64 text-base-content">
				{#if chatStore.status === 'paired'}
					ペアリング完了 (相互通信中)
				{:else if chatStore.status === 'waiting'}
					ペアリング相手を待機中...
				{:else if chatStore.status === 'connecting'}
					接続中...
				{:else}
					未接続
				{/if}
			</span>
		</div>

		{#if chatStore.languageNotSelected}
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
            <option value="ga">Irish</option>
            <option value="nl">Dutch</option>
            <option value="de">German</option>
            <option value="fr">French</option>
            <option value="it">Italian</option>
            <option value="es">Spanish</option>
            <option value="pt">Portuguese</option>
            <option value="ko">Korean</option>
            <option value="ms">Malay</option>
            <option value="fil">Filipino</option>
            <option value="id">Indonesian</option>
            <option value="th">Thai</option>
		</select>

		<div class="flex min-w-64 justify-end">
			{#if chatStore.status === 'disconnected'}
				<button class="btn btn-outline btn-info" onclick={handleConnect}> 接続する </button>
			{:else}
				<button class="btn btn-outline btn-secondary" onclick={() => chatStore.disconnect()}>
					{#if chatStore.status === 'waiting'}
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
			<div class="">
				<span class="text-[0.85rem] font-semibold text-base-content">相手のメッセージ</span>
			</div>
			<div class="flex items-center rounded-md bg-base-200 break-all text-base-content">
				{#if chatStore.partnerText}
					<span
						class="min-h-11 w-full rounded-md border border-zinc-300 p-14 text-base transition-all duration-200"
					>
						{chatStore.partnerText}
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
				bind:value={chatStore.inputText}
				disabled={!chatStore.isPaired}
				oninput={() => chatStore.sendMessage()}
				placeholder={chatStore.isPaired
					? 'メッセージを入力してください...'
					: '接続してペアリングすると入力できます'}
			/>
		</div>
	</div>
</main>

<footer class="mt-3 items-center justify-center text-center">
	<div class="mt-3 flex justify-center gap-32">
		<button class="btn" onclick={() => termsOfServiceModal?.showModal()}>利用規約</button>
		<button class="btn" onclick={() => privacyPolicyModal?.showModal()}>プライバシーポリシー</button>
		<button class="btn" onclick={() => creditModal?.showModal()}>クレジット</button>
	</div>
	<p class="mt-4">© 2026 Vanit by hino-rs. All Rights Reserved.</p>
</footer>
