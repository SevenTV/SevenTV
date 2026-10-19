<script lang="ts">
	import { ArrowSquareOut, Clipboard, Minus, PencilSimple } from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import mouseTrap from "$/lib/mouseTrap";
	import type { Emote, EmoteSetEmote } from "$/gql/graphql";
	import { type EmoteSet } from "$/gql/graphql";
	import { page } from "$app/stores";
	import RenameEmoteDialog from "./dialogs/rename-emote-in-set-dialog.svelte";
	import { removeEmoteFromSet, addEmoteToSet } from "$/lib/setMutations";
	import { editableEmoteSets } from "$/lib/emoteSets";
	import { fade } from "svelte/transition";
	import type { DialogMode } from "./dialogs/dialog.svelte";
	import { user } from "$/lib/auth";

	interface Props {
		currentEmoteSelected?: Emote | EmoteSetEmote;
		data: Emote | EmoteSetEmote;
		position?: { x: number; y: number };
		emoteSetForContextMenu?: EmoteSet | null;
		hideEmoteDialogMode?: () => void;
	}

	let {
		data,
		position = $bindable(),
		emoteSetForContextMenu,
		hideEmoteDialogMode,
		currentEmoteSelected
	}: Props = $props();

	let renameEmoteDialogMode = $state<DialogMode>("hidden");

	function hide() {
		position = undefined;
	}

	function copyLink() {
		navigator.clipboard.writeText(new URL(`/emotes/${data.id}`, $page.url).href);
		hide();
	}

	function showRenameEmoteDialog() {
		renameEmoteDialogMode = "shown";
	}

	let currentEditingSet = $derived(
		$editableEmoteSets.find((set) => set.id === emoteSetForContextMenu?.id),
	);
	let emoteSetName = $derived(emoteSetForContextMenu?.name ?? "");
	let emoteSetId = $derived(emoteSetForContextMenu?.id ?? "");
</script>

<RenameEmoteDialog
	bind:mode={renameEmoteDialogMode}
	{data}
	{emoteSetName}
	{emoteSetId}
	hideFromRenameDialog={hideEmoteDialogMode}
/>

{#if position}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="contextmenu-container" transition:fade={{ duration: 100 }}>
		<nav
			class="contextmenu"
			use:mouseTrap={hide}
			style:left="{position.x}px"
			style:top="{position.y}px"
		>
			{#if $user}
			<Button
					big
					onclick={async () => {
						// replace emote set with new one
						// let currentEditingSet = $editableEmoteSets.find(
						// 	(set) => set.id === emoteSetForContextMenu?.id,
						// );
						let emoteInSet = currentEditingSet?.emotes.items.find((e) => e.emote?.id === data?.id);
						let idToRemove = emoteInSet ? emoteInSet.id : null;
						if (emoteInSet) {
							await removeEmoteFromSet(emoteSetForContextMenu!.id, idToRemove!);
							await addEmoteToSet(emoteSetForContextMenu!.id, currentEmoteSelected!.id);
						}	
						hideEmoteDialogMode?.();
						hide();
					}}
				>
					{#snippet icon()}
						<Minus />
					{/snippet}
					Replace Emote in Set
				</Button>
				<Button
					big
					onclick={async () => {
						// let currentEditingSet = $editableEmoteSets.find(
						// 	(set) => set.id === emoteSetForContextMenu?.id,
						// );
						let emoteInSet = currentEditingSet?.emotes.items.find((e) => e.emote?.id === data?.id);
						let idToRemove = emoteInSet ? emoteInSet.id : null;
						if (emoteInSet) {
							await removeEmoteFromSet(emoteSetForContextMenu!.id, idToRemove!);
							hideEmoteDialogMode?.();
							hide();
						}
					}}
				>
					{#snippet icon()}
						<Minus />
					{/snippet}
					Remove Emote from Set
				</Button>

				<Button
					big
					onclick={() => {
						showRenameEmoteDialog();
						hide();
					}}
				>
					{#snippet icon()}
						<PencilSimple />
					{/snippet}
					Rename Emote in Set
				</Button>
			{/if}
			<Button big href="/emotes/{data.id}" target="_blank" onclick={hide}>
				{#snippet icon()}
					<ArrowSquareOut />
				{/snippet}
				Open in New Tab
			</Button>
			<Button big onclick={copyLink}>
				{#snippet icon()}
					<Clipboard />
				{/snippet}
				Copy Emote Link
			</Button>
		</nav>
	</div>
{/if}

<style lang="scss">
	.contextmenu-container {
		position: fixed;
		top: 0;
		left: 0;
		bottom: 0;
		right: 0;

		z-index: 100;
		pointer-events: all;

		.contextmenu {
			position: absolute;

			border-radius: 0.5rem;

			display: flex;
			flex-direction: column;
			background-color: var(--bg-light);
		}
	}
</style>
