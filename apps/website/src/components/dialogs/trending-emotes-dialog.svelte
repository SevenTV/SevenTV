<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { type Image, type SortBy } from "$/gql/graphql";
	import ResponsiveImage from "../responsive-image.svelte";
	import Spinner from "../spinner.svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		sortBy: SortBy;
		title: string;
	}

	let { mode = $bindable("hidden"), sortBy, title }: Props = $props();

	type TopEmoteItem = { id: string; defaultName: string; images: Image[] };

	// Server allows up to perPage: 250 / page: 100 for this query (see EmoteQuery::search
	// in apps/api). 100 per page keeps a single request quick while still allowing "Load more"
	// to page through everything the server has for this ranking category.
	const PER_PAGE = 100;

	let items = $state<TopEmoteItem[]>([]);
	let page = $state(1);
	let loading = $state(false);
	let loadError = $state(false);
	let hasMore = $state(true);

	async function fetchPage(sortBy: SortBy, page: number): Promise<TopEmoteItem[]> {
		const res = await gqlClient()
			.query(
				graphql(`
					query TrendingEmotesList($sortBy: SortBy!, $page: Int!, $perPage: Int!) {
						emotes {
							search(sort: { sortBy: $sortBy, order: DESCENDING }, page: $page, perPage: $perPage) {
								items {
									id
									defaultName
									images {
										url
										mime
										size
										width
										height
										scale
										frameCount
									}
								}
							}
						}
					}
				`),
				{ sortBy, page, perPage: PER_PAGE },
			)
			.toPromise();

		if (res.error || !res.data) {
			throw res.error;
		}

		return res.data.emotes.search.items;
	}

	async function loadFirstPage(sortBy: SortBy) {
		items = [];
		page = 1;
		hasMore = true;
		loadError = false;
		loading = true;

		try {
			const results = await fetchPage(sortBy, 1);
			items = results;
			hasMore = results.length === PER_PAGE;
		} catch (error) {
			console.error("Failed to load trending emotes list", error);
			loadError = true;
		} finally {
			loading = false;
		}
	}

	async function loadMore() {
		if (loading || !hasMore) return;

		loading = true;
		const nextPage = page + 1;

		try {
			const results = await fetchPage(sortBy, nextPage);
			items = [...items, ...results];
			page = nextPage;
			hasMore = results.length === PER_PAGE;
		} catch (error) {
			console.error("Failed to load more trending emotes", error);
			hasMore = false;
		} finally {
			loading = false;
		}
	}

	// (Re)load whenever the dialog is opened, and whenever it's reused for a different
	// ranking category (trending week vs. top all-time) while already open.
	$effect(() => {
		if (mode !== "hidden") {
			loadFirstPage(sortBy);
		}
	});
</script>

<Dialog width={26} bind:mode>
	<div class="layout">
		<h1>{title}</h1>
		<hr />
		<div class="list">
			{#each items as emote, index}
				<a href="/emotes/{emote.id}" class="row">
					<span class="rank">#{index + 1}</span>
					<ResponsiveImage images={emote.images} width={32} height={32} />
					<span class="name">{emote.defaultName}</span>
				</a>
			{/each}
			{#if !loading && !loadError && items.length === 0}
				<p class="empty">{$t("pages.emote.info.trending_list_empty")}</p>
			{/if}
			{#if loading}
				<div class="spinner-wrapper">
					<Spinner />
				</div>
			{/if}
			{#if loadError}
				<p class="empty error">{$t("pages.emote.info.trending_list_error")}</p>
			{/if}
		</div>
		{#if !loading && hasMore && items.length > 0}
			<Button secondary onclick={loadMore}>{$t("pages.emote.info.trending_list_load_more")}</Button>
		{/if}
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.75rem;

		max-height: 80vh;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		overflow-y: auto;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 0.75rem;

		padding: 0.4rem 0.5rem;
		border-radius: 0.35rem;
		color: var(--text);
		text-decoration: none;

		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}
	}

	.rank {
		width: 2.5rem;
		flex-shrink: 0;
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--text-light);
	}

	.name {
		font-size: 0.85rem;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.spinner-wrapper {
		text-align: center;
		padding: 1rem 0;
	}

	.empty {
		color: var(--text-light);
		font-size: 0.85rem;
		text-align: center;
		padding: 1rem 0;
	}

	.empty.error {
		color: var(--danger);
	}
</style>
