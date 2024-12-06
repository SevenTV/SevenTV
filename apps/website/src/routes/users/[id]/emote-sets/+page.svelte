<script lang="ts">
	import EmoteSetPreview from "$/components/emote-set-preview.svelte";
	import Spinner from "$/components/spinner.svelte";
	import type { PageData } from "./$types";
	import DefaultEmoteSetButton from "$/components/default-emote-set-button.svelte";
	import Button from "$/components/input/button.svelte";
	import { Plus } from "phosphor-svelte";
	import CreateEmoteSetDialog from "$/components/dialogs/create-emote-set-dialog.svelte";
	import type { DialogMode } from "$/components/dialogs/dialog.svelte";

	let { data }: { data: PageData } = $props();

	let createEmoteSetDialog: DialogMode = $state("hidden");
</script>

<div class="buttons">
	<CreateEmoteSetDialog bind:mode={createEmoteSetDialog} />
	<DefaultEmoteSetButton />
	<Button primary onclick={() => (createEmoteSetDialog = "shown")}>
		{#snippet icon()}
			<Plus />
		{/snippet}
		Create new set
	</Button>
</div>
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
	.buttons {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		justify-content: space-between;
	}

	.spinner-wrapper {
		margin: 0 auto;
	}

	.emote-sets {
		display: grid;
		gap: 1rem;
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
	}
</style>
