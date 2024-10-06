<script lang="ts">
	import type { EmoteSearchResult } from "$/gql/graphql";
	import { emotesLayout, Layout } from "$/store/layout";
	import { getContextClient, type Client } from "@urql/svelte";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import InfiniteLoading, { type InfiniteEvent } from "svelte-infinite-loading";

	const PER_PAGE = 36;

	export let load: (client: Client, page: number, perPage: number) => Promise<EmoteSearchResult>;

	let page = 1;
	let results: EmoteSearchResult | null = null;

	function reset() {
		page = 1;
		results = null;
	}

	$: load, reset();

	const client = getContextClient();

	function handleInfinite(e: InfiniteEvent) {
		load(client, page++, PER_PAGE).then((result) => {
			if (results) {
				results.pageCount = result.pageCount;
				results.totalCount = result.totalCount;
				results.items.push(...result.items);
			} else {
				results = result;
			}

			if (results.items.length > 0) {
				e.detail.loaded();
			}

			if (results.pageCount <= page) {
				e.detail.complete();
			}
		});
	}
</script>

<EmoteContainer scrollable layout={$emotesLayout} style="flex-grow: 1">
	{#if results}
		{#each results.items as data, i}
			<EmotePreview {data} index={i} emoteOnly={$emotesLayout === Layout.SmallGrid} />
		{/each}
	{/if}
	<div class="spinner">
		<InfiniteLoading identifier={load} on:infinite={handleInfinite} spinner="spiral">
			<p slot="noMore">No more emotes</p>
			<p slot="noResults">No emotes</p>
		</InfiniteLoading>
	</div>
</EmoteContainer>

<style lang="scss">
	.spinner {
		grid-column: 1 / -1;
	}
</style>
