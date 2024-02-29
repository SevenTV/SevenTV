<script lang="ts">
	import EmoteContainer from "$/components/emote-container.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Bookmarked Emotes - 7TV</title>
</svelte:head>

<h2>Bookmarked Emotes</h2>
<EmoteContainer scrollable>
	{#each Array(100) as _, i}
		<EmotePreview index={i} name="bookmarkedEmote{i}" />
	{/each}
</EmoteContainer>

<style lang="scss">
	h2 {
		font-size: 1.125rem;
		font-weight: 600;
	}
</style>
