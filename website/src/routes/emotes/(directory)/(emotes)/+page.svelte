<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.trending_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#each Array(100) as _, i}
	<EmotePreview
		index={i}
		name="trending{i}"
		emoteOnly={$emotesLayout === Layout.SmallGrid}
		ignoredFlagsForHighlight={["trending"]}
	/>
{/each}
