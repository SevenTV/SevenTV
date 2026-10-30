<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import TrendingEmotesDialog from "$/components/dialogs/trending-emotes-dialog.svelte";
	import { type DialogMode } from "$/components/dialogs/dialog.svelte";
	import type { PageData } from "./$types";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { SortBy, type EmoteScores } from "$/gql/graphql";
	import { t } from "svelte-i18n";
	import { numberFormat, exactNumberFormat } from "$/lib/utils";
	import { Users, ChartLineUp, Trophy } from "phosphor-svelte";

	let { data }: { data: PageData } = $props();

	let dialogMode: DialogMode = $state("hidden");
	let dialogSortBy: SortBy = $state(SortBy.TrendingWeekly);
	let dialogTitle = $state("");

	function openTrendingDialog(sortBy: SortBy, title: string) {
		dialogSortBy = sortBy;
		dialogTitle = title;
		dialogMode = "shown";
	}

	type InfoStats = {
		channelCount: number;
		scores: EmoteScores;
		trendingWeekRank: number | null;
		topAllTimeRank: number | null;
	};

	async function loadStats(id: string): Promise<InfoStats> {
		try {
			return await loadStatsUnsafe(id);
		} catch (error) {
			console.error("Failed to load emote info stats", error);
			throw error;
		}
	}

	async function loadStatsUnsafe(id: string): Promise<InfoStats> {
		const res = await gqlClient()
			.query(
				graphql(`
					query EmoteInfoStats($id: Id!) {
						emotes {
							emote(id: $id) {
								channels(page: 1, perPage: 1) {
									totalCount
								}
								scores {
									topAllTime
									topDaily
									topWeekly
									topMonthly
									trendingDay
									trendingWeek
									trendingMonth
								}
							}
						}
					}
				`),
				{ id },
			)
			.toPromise();

		if (res.error || !res.data || !res.data.emotes.emote) {
			throw res.error;
		}

		const scores = res.data.emotes.emote.scores;

		// compute the displayed rank from the exact same live-sorted list the "view list"
		// dialog uses, instead of the separately (once-a-day) cached ranking() field, so the
		// number shown here always matches what you see when you click through to the list
		const [trendingWeekRank, topAllTimeRank] = await Promise.all([
			scores.trendingWeek > 0 ? findLiveRank(SortBy.TrendingWeekly, id) : null,
			scores.topAllTime > 0 ? findLiveRank(SortBy.TopAllTime, id) : null,
		]);

		return {
			channelCount: res.data.emotes.emote.channels.totalCount,
			scores,
			trendingWeekRank,
			topAllTimeRank,
		};
	}

	// server allows perPage up to 250 (see EmoteQuery::search in apps/api). Two pages covers
	// roughly the same "top ~500" scope the old ranking() cap had; beyond that we just show
	// "not ranked" rather than paging indefinitely to find an exact number.
	const RANK_LOOKUP_PER_PAGE = 250;
	const RANK_LOOKUP_MAX_PAGES = 2;

	async function findLiveRank(sortBy: SortBy, targetId: string): Promise<number | null> {
		for (let page = 1; page <= RANK_LOOKUP_MAX_PAGES; page++) {
			const res = await gqlClient()
				.query(
					graphql(`
						query EmoteLiveRankLookup($sortBy: SortBy!, $page: Int!, $perPage: Int!) {
							emotes {
								search(
									sort: { sortBy: $sortBy, order: DESCENDING }
									page: $page
									perPage: $perPage
								) {
									items {
										id
									}
								}
							}
						}
					`),
					{ sortBy, page, perPage: RANK_LOOKUP_PER_PAGE },
				)
				.toPromise();

			if (res.error || !res.data) {
				throw res.error;
			}

			const items = res.data.emotes.search.items;
			const index = items.findIndex((item) => item.id === targetId);

			if (index !== -1) {
				return (page - 1) * RANK_LOOKUP_PER_PAGE + index + 1;
			}

			if (items.length < RANK_LOOKUP_PER_PAGE) {
				break;
			}
		}

		return null;
	}

	let stats = $derived(loadStats(data.id));
</script>

<div class="navigation">
	{#await data.streamed.emote then emote}
		<EmoteTabs id={emote.id} />
	{/await}
</div>

<div class="stats">
	{#await stats}
		<div class="stat loading-animation"></div>
		<div class="stat loading-animation"></div>
	{:then stats}
		<div class="stat" title={$t("pages.emote.info.channels_tooltip")}>
			<Users size="1.25rem" />
			<div>
				<span class="value">{exactNumberFormat().format(stats.channelCount)}</span>
				<span class="label">{$t("pages.emote.info.used_by_channels")}</span>
			</div>
		</div>
		{#if stats.trendingWeekRank !== null || (stats.scores?.trendingWeek ?? 0) > 0}
			<button
				type="button"
				class="stat clickable"
				title={$t("pages.emote.info.trending_tooltip")}
				onclick={() =>
					openTrendingDialog(SortBy.TrendingWeekly, $t("pages.emote.info.trending_this_week"))}
			>
				<ChartLineUp size="1.25rem" />
				<div>
					<span class="value">
						{#if stats.trendingWeekRank !== null}
							#{numberFormat().format(stats.trendingWeekRank)}
						{:else}
							{$t("pages.emote.info.not_ranked")}
						{/if}
					</span>
					<span class="label">{$t("pages.emote.info.trending_this_week")}</span>
				</div>
			</button>
		{/if}
		{#if stats.topAllTimeRank !== null || (stats.scores?.topAllTime ?? 0) > 0}
			<button
				type="button"
				class="stat clickable"
				title={$t("pages.emote.info.top_all_time_tooltip")}
				onclick={() => openTrendingDialog(SortBy.TopAllTime, $t("pages.emote.info.top_all_time"))}
			>
				<Trophy size="1.25rem" />
				<div>
					<span class="value">
						{#if stats.topAllTimeRank !== null}
							#{numberFormat().format(stats.topAllTimeRank)}
						{:else}
							{$t("pages.emote.info.not_ranked")}
						{/if}
					</span>
					<span class="label">{$t("pages.emote.info.top_all_time")}</span>
				</div>
			</button>
		{/if}
	{:catch}
		<p class="stats-error">{$t("pages.emote.info.stats_error")}</p>
	{/await}
</div>

<p class="usage-note">{$t("pages.emote.info.usage_note")}</p>

<TrendingEmotesDialog bind:mode={dialogMode} sortBy={dialogSortBy} title={dialogTitle} />

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.stats {
		margin-top: 1.5rem;

		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;

		.stat {
			flex: 1 1 10rem;
			min-height: 3.5rem;

			display: flex;
			align-items: center;
			gap: 0.75rem;

			padding: 0.75rem 1rem;
			background-color: var(--bg-light);
			border-radius: 0.5rem;
			color: var(--primary);
			cursor: help;

			&.clickable {
				border: none;
				font: inherit;
				text-align: left;
				cursor: pointer;

				&:hover,
				&:focus-visible {
					background-color: var(--bg-medium);
				}
			}

			div {
				display: flex;
				flex-direction: column;
			}

			.value {
				font-size: 1rem;
				font-weight: 700;
				color: var(--text);
			}

			.label {
				font-size: 0.75rem;
				color: var(--text-light);
			}
		}
	}

	.usage-note {
		margin-top: 0.75rem;
		font-size: 0.75rem;
		color: var(--text-light);
	}

	.stats-error {
		margin-top: 0.75rem;
		font-size: 0.75rem;
		color: var(--danger);
	}
</style>
