<script lang="ts">
	import type { LayoutData } from "./$types";
	import { t } from "svelte-i18n";
	import EmoteInfo from "$/components/emotes/emote-info.svelte";
	import type { Snippet } from "svelte";
	import { invalidate } from "$app/navigation";
	import { browser } from "$app/environment";
	import type { Emote } from "$/gql/graphql";
	import { WifiX } from "phosphor-svelte";

	let { data, children }: { data: LayoutData; children: Snippet } = $props();

	let loadFailed = $state(false);
	let isOnline = $state(browser ? navigator.onLine : true);

	$effect(() => {
		loadFailed = false;
		data.streamed.emote.then((emote: Emote | undefined) => {
			if (!emote) loadFailed = true;
		});
	});

	$effect(() => {
		if (!browser) return;

		function handleOnline() {
			isOnline = true;
			if (loadFailed) {
				invalidate(`emotes:${data.id}`);
			}
		}

		function handleOffline() {
			isOnline = false;
		}

		window.addEventListener("online", handleOnline);
		window.addEventListener("offline", handleOffline);
		return () => {
			window.removeEventListener("online", handleOnline);
			window.removeEventListener("offline", handleOffline);
		};
	});
</script>

<svelte:head>
	{#await data.streamed.emote}
		<title>Loading - {$t("page_titles.suffix")}</title>
	{:then emote}
		<title>{emote?.defaultName ?? $t("page_titles.suffix")} - {$t("page_titles.suffix")}</title>
	{/await}
</svelte:head>

{#if !isOnline && loadFailed}
	<div class="offline-banner">
		<WifiX size={18} />
		{$t("common.offline_retry")}
	</div>
{/if}

<div class="layout">
	<div>
		{#await data.streamed.emote}
			<EmoteInfo data={null} />
		{:then emote}
			<EmoteInfo data={emote ?? null} />
		{/await}
	</div>
	<div>
		{@render children()}
	</div>
</div>

<style lang="scss">
	.offline-banner {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		max-width: 80rem;
		margin-inline: auto;
		margin-top: 0.75rem;
		padding: 0.6rem 1rem;
		background-color: var(--bg-medium);
		border: 1px solid var(--danger);
		border-radius: 0.5rem;
		color: var(--danger);
		font-size: 0.875rem;
	}

	.layout {
		width: 100%;
		max-width: 80rem;
		margin-inline: auto;

		padding: 1.25rem;
		min-height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		& > * {
			background-color: var(--bg-medium);
			border: 1px solid var(--layout-border);
			border-radius: 0.5rem;
			padding: 1rem;
		}
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
		}
	}
</style>
