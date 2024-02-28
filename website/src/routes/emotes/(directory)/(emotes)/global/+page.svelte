<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Global Emotes - 7TV</title>
</svelte:head>

{#each Array(10) as _, i}
	<EmotePreview index={i} name="globalEmote{i}" />
{/each}
