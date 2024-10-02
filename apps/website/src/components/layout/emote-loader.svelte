<script lang="ts">
	import type { Emote } from "$/gql/graphql";
	import EmoteLoadingPlaceholder from "../emote-loading-placeholder.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import { Layout, emotesLayout } from "$/store/layout";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
	import Button from "../input/button.svelte";

	const MAX_PAGES = 100;
	const MAX_LIMIT = 250;

	export let load: (page: number, limit: number) => Promise<Emote[]>;
	export let page = 1;

	let containerWidth: number | undefined;
	let containerHeight: number | undefined;

	$: emoteSize = ($emotesLayout === Layout.SmallGrid) ? (5 * 16) : (10 * 16);
	const EMOTE_GAP = 1 * 16;

	$: limit = containerWidth
		? containerHeight
			? Math.min(Math.floor(containerHeight / (emoteSize + EMOTE_GAP)) *
				Math.floor(containerWidth / (emoteSize + EMOTE_GAP)), MAX_LIMIT)
			: undefined
		: undefined;

	$: results = limit ? load(page, limit) : [];
</script>

<EmoteContainer
	scrollable
	layout={$emotesLayout}
	style="flex-grow: 1"
	bind:clientWidth={containerWidth}
	bind:clientHeight={containerHeight}
>
	{#await results}
		{#each Array(limit ?? 36) as _, i}
			<EmoteLoadingPlaceholder index={i} />
		{/each}
	{:then results}
		{#each results as result, i}
			<EmotePreview index={i} data={result} emoteOnly={$emotesLayout === Layout.SmallGrid} />
		{/each}
	{/await}
</EmoteContainer>
<div class="buttons">
	<Button disabled={page <= 1} on:click={() => page--}>
		<CaretLeft slot="icon" />
		Previous Page
	</Button>
	<span>Page {page} of {MAX_PAGES}</span>
	<Button disabled={page >= MAX_PAGES} on:click={() => page++}>
		<CaretRight slot="icon-right" />
		Next Page
	</Button>
</div>

<style lang="scss">
	.buttons {
		display: flex;
		justify-content: center;
		align-items: center;
		flex-wrap: wrap;
		column-gap: 1rem;
	}
</style>
