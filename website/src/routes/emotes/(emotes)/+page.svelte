<script>
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Trending Emotes - 7TV</title>
</svelte:head>

<div class="emotes">
	{#each Array(100) as _, i}
		<EmotePreview index={i} name="trending{i}" />
	{/each}
</div>

<style lang="scss">
	.emotes {
		margin-top: 2rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		justify-items: center;
		align-items: center;
		gap: 1rem;
	}
</style>
