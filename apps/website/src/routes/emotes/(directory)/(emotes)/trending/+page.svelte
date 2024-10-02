<script lang="ts">
	import { SortBy } from "$/gql/graphql";
	import { t } from "svelte-i18n";
	import type { PageData } from "./$types";
	import { queryEmotes } from "$/lib/emoteQuery";
	import EmoteLoader from "$/components/layout/emote-loader.svelte";

	export let data: PageData;
</script>

<svelte:head>
	<title>{$t("page_titles.trending_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<EmoteLoader
	load={(page, limit) =>
		queryEmotes(data.query, data.tags, SortBy.TrendingWeekly, data.filters, page, limit)}
	page={data.page ?? 1}
/>
