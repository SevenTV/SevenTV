<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.new_emotes")} - 7TV</title>
</svelte:head>

{#each Array(100) as _, i}
	<EmotePreview index={i} name="newEmote{i}" emoteOnly={$emotesLayout === Layout.SmallGrid} />
{/each}
