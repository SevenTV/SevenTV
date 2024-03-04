<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
</script>

<svelte:head>
	<title>Global Emotes - 7TV</title>
</svelte:head>

{#each Array(10) as _, i}
	<EmotePreview index={i} name="globalEmote{i}" emoteOnly={$emotesLayout === Layout.SmallGrid} />
{/each}
