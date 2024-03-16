<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.global_emotes")} - 7TV</title>
</svelte:head>

{#each Array(10) as _, i}
	<EmotePreview index={i} name="globalEmote{i}" emoteOnly={$emotesLayout === Layout.SmallGrid} />
{/each}
