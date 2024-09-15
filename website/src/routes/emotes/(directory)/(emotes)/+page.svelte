<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import { Layout, emotesLayout } from "$/store/layout";
	import { t } from "svelte-i18n";
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

<svelte:head>
	<title>{$t("page_titles.trending_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#each data.results as result, i}
	<EmotePreview
		index={i}
		data={result}
		emoteOnly={$emotesLayout === Layout.SmallGrid}
		ignoredFlagsForHighlight={["trending"]}
	/>
{/each}
