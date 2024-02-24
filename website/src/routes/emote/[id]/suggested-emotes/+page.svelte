<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<div class="emotes">
	{#each new Array(10) as _, i}
		<EmotePreview index={i} name="suggestedEmote{i}" bg="light" />
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

	@media screen and (max-width: 960px) {
		.emotes {
			grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));
		}
	}
</style>
