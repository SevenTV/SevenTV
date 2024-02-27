<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<div class="emotes">
	{#each Array(10) as _, i}
		<EmotePreview index={i} name="suggestedEmote{i}" bg="light" />
	{/each}
</div>

<style lang="scss">
	.emotes {
		margin-top: 1rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		place-items: center;
		gap: 1rem;
	}
</style>
