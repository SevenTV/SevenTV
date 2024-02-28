<script lang="ts">
	import EmoteContainer from "$/components/emote-container.svelte";
import EmotePreview from "$/components/emote-preview.svelte";
</script>

<EmoteContainer>
	{#each Array(100) as _, i}
		<EmotePreview name="uploadedEmote1" index={i} />
	{/each}
</EmoteContainer>
