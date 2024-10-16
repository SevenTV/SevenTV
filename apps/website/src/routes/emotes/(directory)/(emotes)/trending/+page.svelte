<script lang="ts">
	import { SortBy } from "$/gql/graphql";
	import { t } from "svelte-i18n";
	import type { PageData } from "./$types";
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import { queryEmotes } from "$/lib/emoteQuery";

	export let data: PageData;
</script>

<svelte:head>
	<title>{$t("page_titles.trending_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<EmoteLoader
	load={(client, page, perPage) =>
		queryEmotes(client, data.query, data.tags, SortBy.TrendingWeekly, data.filters, page, perPage)}
/>
