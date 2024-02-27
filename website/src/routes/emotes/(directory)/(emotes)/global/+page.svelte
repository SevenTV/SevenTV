<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Global Emotes - 7TV</title>
</svelte:head>

<div class="emotes">
	{#each Array(10) as _, i}
		<EmotePreview index={i} name="globalEmote{i}" />
	{/each}
</div>

<style lang="scss">
	.emotes {
		margin-top: 2rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		place-items: center;
		gap: 1rem;
	}
</style>
