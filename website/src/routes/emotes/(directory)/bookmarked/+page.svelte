<script lang="ts">
	import EmoteContainer from "$/components/emote-container.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Bookmarked Emotes - 7TV</title>
</svelte:head>

<EmoteContainer scrollable>
	{#each Array(100) as _, i}
		<EmotePreview index={i} name="bookmarkedEmote{i}" />
	{/each}
</EmoteContainer>
