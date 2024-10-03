<script lang="ts">
	import type { EmoteSearchResult } from "$/gql/graphql";
	import EmoteLoadingPlaceholder from "../emote-loading-placeholder.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import { Layout, emotesLayout } from "$/store/layout";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { goto } from "$app/navigation";
	import { page } from "$app/stores";

	const MAX_PER_PAGE = 250;

	export let load: (page: number, perPage: number) => Promise<EmoteSearchResult>;
	export let updateUrl = false;
	export let numPage = 1;

	let containerWidth: number | undefined;
	let containerHeight: number | undefined;

	$: emoteSize = $emotesLayout === Layout.SmallGrid ? 5 * 16 : 10 * 16;
	const EMOTE_GAP = 1 * 16;

	$: perPage = calculatePerPage(containerWidth, containerHeight, emoteSize);

	function calculatePerPage(width: number | undefined, height: number | undefined, emoteSize: number) {
		if (!width || !height) {
			return undefined;
		}

		const rows = Math.floor(width / (emoteSize + EMOTE_GAP));
		const columns = Math.floor(height / (emoteSize + EMOTE_GAP));

		return Math.min(rows * columns, MAX_PER_PAGE);
	}

	$: results = perPage
		? load(numPage, perPage).then((result) => {
				pageCount = result.pageCount;

				if (numPage > pageCount) {
					numPage = pageCount;
				}

				return result;
			})
		: null;

	let pageCount: number | null = null;

	$: if (updateUrl) {
		let url = new URL($page.url);

		if (numPage && numPage > 1) {
			url.searchParams.set("p", numPage.toString());
		} else {
			url.searchParams.delete("p");
		}

		goto(url, { replaceState: true, noScroll: true, keepFocus: true });
	}
</script>

<EmoteContainer
	scrollable
	layout={$emotesLayout}
	style="flex-grow: 1"
	bind:clientWidth={containerWidth}
	bind:clientHeight={containerHeight}
>
	{#await results}
		{#each Array(perPage ?? 36) as _, i}
			<EmoteLoadingPlaceholder index={i} />
		{/each}
	{:then results}
		{#if results}
			{#each results.items as data, i}
				<EmotePreview index={i} {data} emoteOnly={$emotesLayout === Layout.SmallGrid} />
			{/each}
		{/if}
	{/await}
</EmoteContainer>
<div class="buttons">
	<Button disabled={numPage <= 1} on:click={() => numPage--} hideOnMobile>
		<CaretLeft slot="icon" />
		Previous Page
	</Button>
	<Button disabled={numPage <= 1} on:click={() => numPage--} hideOnDesktop>
		<CaretLeft slot="icon" />
	</Button>
	<span>
		Page {numPage}
		{#if pageCount}
			of {pageCount}
		{/if}
	</span>
	<Button disabled={!pageCount || numPage >= pageCount} on:click={() => numPage++} hideOnMobile>
		<CaretRight slot="icon-right" />
		Next Page
	</Button>
	<Button disabled={!pageCount || numPage >= pageCount} on:click={() => numPage++} hideOnDesktop>
		<CaretRight slot="icon" />
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
