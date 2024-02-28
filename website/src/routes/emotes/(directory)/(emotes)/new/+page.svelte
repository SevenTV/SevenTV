<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
</script>

<svelte:head>
	<title>New Emotes - 7TV</title>
</svelte:head>

{#each Array(100) as _, i}
	<EmotePreview index={i} name="newEmote{i}" />
{/each}
