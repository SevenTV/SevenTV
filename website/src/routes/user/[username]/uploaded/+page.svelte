<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<div class="emotes">
	{#each Array(100) as _, i}
		<EmotePreview name="uploadedEmote1" index={i} />
	{/each}
</div>

<style lang="scss">
	.emotes {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(var(--emote-preview-size), 1fr));
		gap: 1rem;
	}
</style>
