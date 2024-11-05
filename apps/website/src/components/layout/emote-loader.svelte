<script lang="ts">
	import type { EmoteSearchResult } from "$/gql/graphql";
	import { emotesLayout } from "$/lib/layout";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import InfiniteLoading, { type InfiniteEvent } from "svelte-infinite-loading";
	import { isMobileLayout } from "$/lib/utils";
	import Spinner from "../spinner.svelte";

	const PER_PAGE = 36;

	interface Props {
		load: (page: number, perPage: number) => Promise<EmoteSearchResult>;
	}

	let { load }: Props = $props();

	let page = $state(1);
	let results: EmoteSearchResult | undefined = $state();

	let identifier = $state(0);

	export function reset() {
		page = 1;
		results = undefined;
		identifier++;
	}

	function handleInfinite(event: InfiniteEvent) {
		load(page++, PER_PAGE)
			.then((result) => {
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
			})
			.catch(() => {
				event.detail.error();
			});
	}
</script>

<EmoteContainer scrollable={!isMobileLayout()} layout={$emotesLayout} style="flex-grow: 1">
	{#if results}
		{#each results.items as data, i}
			<EmotePreview {data} index={i} emoteOnly={$emotesLayout === "small-grid"} />
		{/each}
	{/if}
	<div class="loading">
		<!-- Still uses old Svelte 4 slots -->
		<InfiniteLoading
			distance={500}
			identifier={{
				identifier,
				layout: $emotesLayout,
			}}
			on:infinite={handleInfinite}
		>
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
