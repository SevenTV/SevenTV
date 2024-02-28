<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Bookmarked Emotes - 7TV</title>
</svelte:head>

<h2>Bookmarked Emotes</h2>
<div class="emotes">
	{#each Array(100) as _, i}
		<EmotePreview index={i} name="bookmarkedEmote{i}" />
	{/each}
</div>

<style lang="scss">
	h2 {
		font-size: 1.125rem;
		font-weight: 600;
	}

	.emotes {
		margin-top: 1rem;
		overflow: auto;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(var(--emote-preview-size), 1fr));
		place-items: center;
		gap: 1rem;
	}
</style>
