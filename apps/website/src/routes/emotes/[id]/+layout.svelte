<script lang="ts">
	import type { LayoutData } from "./$types";
	import { t } from "svelte-i18n";
	import EmoteInfo from "$/components/emotes/emote-info.svelte";
	import type { Snippet } from "svelte";

	let { data, children }: { data: LayoutData; children: Snippet } = $props();
</script>

<svelte:head>
	{#await data.streamed.emote}
		<title>Loading - {$t("page_titles.suffix")}</title>
	{:then emote}
		<title>{emote.defaultName} - {$t("page_titles.suffix")}</title>
	{/await}
</svelte:head>

<div class="layout">
	<div>
		{#await data.streamed.emote}
			<EmoteInfo data={null} />
		{:then emote}
			<EmoteInfo data={emote} />
		{/await}
	</div>
	<div>
		{@render children()}
	</div>
</div>

<style lang="scss">
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
