<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>Top Emotes - 7TV</title>
</svelte:head>

<div class="emotes">
	{#each new Array(100) as _, i}
		<EmotePreview index={i} name="topEmote{i}" />
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
