<script lang="ts">
	import EmoteSetPreview from "$/components/emote-set-preview.svelte";
	import Spinner from "$/components/spinner.svelte";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();
</script>

{#await data.streamed.sets}
	<div class="spinner-wrapper">
		<Spinner />
	</div>
{:then sets}
	<div class="emote-sets">
		{#each sets as set}
			<EmoteSetPreview data={set} />
		{/each}
	</div>
{/await}

<style lang="scss">
	.spinner-wrapper {
		margin: 0 auto;
	}

	.emote-sets {
		display: grid;
		gap: 1rem;
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
	}
</style>
