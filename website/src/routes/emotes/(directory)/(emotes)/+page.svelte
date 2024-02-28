<script>
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Trending Emotes - 7TV</title>
</svelte:head>

{#each Array(100) as _, i}
	<EmotePreview index={i} name="trending{i}" />
{/each}
