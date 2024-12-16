<script lang="ts">
	import EmoteSetPreview from "$/components/emote-set-preview.svelte";
	import Spinner from "$/components/spinner.svelte";
	import type { PageData } from "./$types";
	import DefaultEmoteSetButton from "$/components/default-emote-set-button.svelte";
	import Button from "$/components/input/button.svelte";
	import { Plus } from "phosphor-svelte";
	import CreateEmoteSetDialog from "$/components/dialogs/create-emote-set-dialog.svelte";
	import type { DialogMode } from "$/components/dialogs/dialog.svelte";
	import { user } from "$/lib/auth";
	import { UserEditorState } from "$/gql/graphql";

	let { data }: { data: PageData } = $props();

	let createEmoteSetDialog: DialogMode = $state("hidden");
</script>

<div class="buttons">
	<CreateEmoteSetDialog ownerId={data.id} bind:mode={createEmoteSetDialog} />
	<DefaultEmoteSetButton />
	{#await data.streamed.userRequest.value then userData}
		{#if $user?.permissions.emoteSet.manage && ($user.permissions.emoteSet.manageAny || $user?.id === data.id || userData.editors.some((editor) => editor.editorId === $user?.id && editor.state === UserEditorState.Accepted && editor.permissions.emoteSet.create))}
			<Button primary onclick={() => (createEmoteSetDialog = "shown")}>
				{#snippet icon()}
					<Plus />
				{/snippet}
				Create New Set
			</Button>
		{/if}
	{/await}
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
