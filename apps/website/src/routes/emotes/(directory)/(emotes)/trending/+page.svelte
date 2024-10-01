<script lang="ts">
	import EmoteLoadingPlaceholder from "$/components/emote-loading-placeholder.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
	import { SortBy } from "$/gql/graphql";
	import { Layout, emotesLayout } from "$/store/layout";
	import { t } from "svelte-i18n";
	import type { PageData } from "./$types";
	import { queryEmotes } from "$/lib/emoteQuery";

	export let data: PageData;

	const limit = 36;

	$: results = queryEmotes(data.query, data.tags, SortBy.TrendingWeekly, data.filters, data.page, limit);
</script>

<svelte:head>
	<title>{$t("page_titles.trending_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#await results}
	{#each Array(limit) as _, i}
		<EmoteLoadingPlaceholder index={i} />
	{/each}
{:then results}
	{#each results as result, i}
		<EmotePreview
			index={i}
			data={result}
			emoteOnly={$emotesLayout === Layout.SmallGrid}
			ignoredFlagsForHighlight={["trending"]}
		/>
	{/each}
{/await}
