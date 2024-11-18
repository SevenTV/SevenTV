<script lang="ts">
	import type { Emote, EmoteSearchResult, EmoteSetEmoteSearchResult } from "$/gql/graphql";
	import { emotesLayout } from "$/lib/layout";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import InfiniteLoading, { type InfiniteEvent } from "svelte-infinite-loading";
	import Spinner from "../spinner.svelte";
	import { untrack } from "svelte";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";

	const PER_PAGE = 72;

	interface Props {
		load: (page: number, perPage: number) => Promise<EmoteSearchResult | EmoteSetEmoteSearchResult>;
		scrollable?: boolean;
		selectionMode?: boolean;
		selectionMap?: { [key: string]: boolean };
	}

	let { load, scrollable, selectionMode = false, selectionMap = $bindable({}) }: Props = $props();

	let page = $state(1);
	let results: EmoteSearchResult | undefined = $state();

	let identifier = $state(0);

	export function reset() {
		page = 1;
		results = undefined;
		identifier++;
	}

	// Reset when the layout or the default emote set changes
	$effect(() => {
		$emotesLayout;
		$defaultEmoteSet;
		untrack(() => {
			reset();
		});
	});

	function handleInfinite(event: InfiniteEvent) {
		load(page++, PER_PAGE)
			.then((result) => {
				// Convert EmoteSetEmoteSearchResult to EmoteSearchResult
				if (result.__typename === "EmoteSetEmoteSearchResult") {
					// @ts-expect-error I know what I'm doing
					result.__typename = "EmoteSearchResult";
					// @ts-expect-error I know what I'm doing
					result.items = result.items
						.filter((item) => item.emote)
						.map((item) => {
							const emote = item.emote!;

							emote.defaultName = item.alias || emote!.defaultName;
							emote.flags.defaultZeroWidth = item.flags.zeroWidth || emote.flags.defaultZeroWidth;

							return emote as Emote;
						}) as Emote[];
				}

				result = result as EmoteSearchResult;

				if (results) {
					results.pageCount = result.pageCount;
					results.totalCount = result.totalCount;
					results.items.push(...result.items);
				} else {
					results = result as EmoteSearchResult;
				}

				if (results.items.length > 0) {
					event.detail.loaded();
				}

				if (results.pageCount <= page) {
					event.detail.complete();
				}

				for (let item of results.items) {
					selectionMap[item.id] = selectionMap[item.id] || false;
				}
			})
			.catch(() => {
				event.detail.error();
			});
	}
</script>

<EmoteContainer {scrollable} layout={$emotesLayout} style="flex-grow: 1">
	{#if results}
		{#each results.items as data, i}
			<EmotePreview
				{data}
				index={i}
				emoteOnly={$emotesLayout === "small-grid"}
				{selectionMode}
				selected={selectionMap[data.id]}
			/>
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
