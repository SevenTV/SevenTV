<script lang="ts">
	import { SortBy } from "$/gql/graphql";
	import { t } from "svelte-i18n";
	import type { PageData } from "./$types";
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import { queryEmotes } from "$/lib/emoteQuery";
	import { untrack } from "svelte";

	let { data }: { data: PageData } = $props();

	// TODO: Find out correct type
	let loader: any;

	$effect(() => {
		data;
		untrack(() => {
			loader?.reset();
		});
	});
</script>

<svelte:head>
	<title>{$t("page_titles.new_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<EmoteLoader
	bind:this={loader}
	load={(client, page, perPage) =>
		queryEmotes(client, data.query, data.tags, SortBy.UploadDate, data.filters, page, perPage)}
/>
