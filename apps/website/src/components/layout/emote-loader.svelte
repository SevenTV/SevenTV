<script lang="ts">
	import type { EmoteSearchResult, Filters, SortBy } from "$/gql/graphql";
	import { emotesLayout, Layout } from "$/store/layout";
	import { getContextClient, type Client } from "@urql/svelte";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import InfiniteLoading, { type InfiniteEvent } from "svelte-infinite-loading";
	import { isMobileLayout } from "$/lib/utils";
	import Spinner from "../spinner.svelte";
	import EmoteLoadingPlaceholder from "../emote-loading-placeholder.svelte";
	import { queryEmotes } from "$/lib/emoteQuery";

	const PER_PAGE = 36;

	interface LoadOptions {
		query: string | null;
		tags: string[];
		sortBy: SortBy;
		filters: Filters;
	};

	// export let load: (client: Client, page: number, perPage: number) => Promise<EmoteSearchResult>;
	export let options: LoadOptions;

	let page = 1;
	let results: EmoteSearchResult | null = null;

	function reset() {
		page = 1;
		results = null;
	}

	$: options, reset();

	const client = getContextClient();

	function handleInfinite(event: InfiniteEvent) {
		queryEmotes(client, options.query, options.tags, options.sortBy, options.filters, page++, PER_PAGE).then((result) => {
			if (results) {
				results.pageCount = result.pageCount;
				results.totalCount = result.totalCount;
				results.items.push(...result.items);
			} else {
				results = result;
			}

			if (results.items.length > 0) {
				event.detail.loaded();
			}

			if (results.pageCount <= page) {
				event.detail.complete();
			}
		}).catch(() => {
			event.detail.error();
		});
	}
</script>

<EmoteContainer scrollable={!isMobileLayout()} layout={$emotesLayout} style="flex-grow: 1">
	{#if results}
		{#each results.items as data, i}
			<EmotePreview {data} index={i} emoteOnly={$emotesLayout === Layout.SmallGrid} />
		{/each}
	{:else}
		{#each Array(PER_PAGE) as _, i}
			<EmoteLoadingPlaceholder index={i} />
		{/each}
	{/if}
	<div class="loading">
		<InfiniteLoading identifier={options} on:infinite={handleInfinite}>
			<p slot="noMore">No more emotes</p>
			<p slot="noResults">No emotes</p>
			<Spinner slot="spinner" />
		</InfiniteLoading>
	</div>
</EmoteContainer>

<style lang="scss">
	.loading {
		grid-column: 1 / -1;
		align-self: start;

		width: 100%;
		height: 1rem;
	}
</style>
