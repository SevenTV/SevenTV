<script lang="ts">
	import { ArrowSquareOut, Clipboard, FolderSimple, NotePencil, Trash } from "phosphor-svelte";
	import AddEmoteDialog from "./dialogs/add-emote-dialog.svelte";
	import RenameEmoteDialog from "./dialogs/rename-emote-in-set-dialog.svelte";
	import RemoveEmoteDialog from "./dialogs/remove-emote-from-set-dialog.svelte";
	import Button from "./input/button.svelte";
	import mouseTrap from "$/lib/mouseTrap";
	import type { DialogMode } from "./dialogs/dialog.svelte";
	import type { Emote, EmoteSetEmote } from "$/gql/graphql";
	import EmoteUseButton from "./emote-use-button.svelte";
	import { page } from "$app/stores";
	import { fade } from "svelte/transition";
	import { user } from "$/lib/auth";
	import { editableEmoteSets } from "$/lib/emoteSets";
	import { t } from "svelte-i18n";

	interface Props {
		data: Emote;
		emoteInSet?: EmoteSetEmote;
		emoteSetName?: string;
		emoteSetId?: string;
		resetComponent?: () => void;
		position?: { x: number; y: number };
	}

	let {
		data,
		emoteInSet,
		emoteSetName,
		emoteSetId,
		resetComponent,
		position = $bindable(),
	}: Props = $props();

	function hide() {
		position = undefined;
	}

	let addEmoteDialogMode = $state<DialogMode>("hidden");
	let renameEmoteDialogMode = $state<DialogMode>("hidden");
	let removeEmoteDialogMode = $state<DialogMode>("hidden");

	const editableSets = $derived($editableEmoteSets?.map((set) => set.id) ?? []);
	const isCurrentSetEditable = $derived(emoteSetId ? editableSets.includes(emoteSetId) : false);

	$effect(() => {
		if (
			addEmoteDialogMode === "hidden" &&
			renameEmoteDialogMode === "hidden" &&
			removeEmoteDialogMode === "hidden"
		) {
			hide();
		}
	});
	function showAddEmoteDialog() {
		addEmoteDialogMode = "shown";
	}

	function showRenameEmoteDialog() {
		renameEmoteDialogMode = "shown";
	}

	function showRemoveEmoteDialog() {
		removeEmoteDialogMode = "shown";
	}

	function copyLink() {
		navigator.clipboard.writeText(new URL(`/emotes/${data.id}`, $page.url).href);
		hide();
	}

	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
	}
</script>

{#if position}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="contextmenu-container"
		transition:fade={{ duration: 100 }}
		oncontextmenu={onContextMenu}
	>
		<nav
			class="contextmenu"
			use:mouseTrap={hide}
			style:left="{position.x}px"
			style:top="{position.y}px"
		>
			{#if $user}
				<EmoteUseButton {data} big oncomplete={hide} />
				<AddEmoteDialog bind:mode={addEmoteDialogMode} {data} />
				{#if emoteSetId && emoteSetName && isCurrentSetEditable}
					<RenameEmoteDialog
						bind:mode={renameEmoteDialogMode}
						{data}
						{emoteInSet}
						{emoteSetId}
						{emoteSetName}
						{resetComponent}
					/>
					<RemoveEmoteDialog
						bind:mode={removeEmoteDialogMode}
						{data}
						{emoteInSet}
						{emoteSetId}
						{emoteSetName}
						{resetComponent}
					/>
				{/if}
				<Button big onclick={showAddEmoteDialog}>
					{#snippet icon()}
						<FolderSimple />
					{/snippet}
					{$t("dialogs.emote_context_menu.add_remove_emote")}
				</Button>
				{#if emoteSetId && emoteSetName && isCurrentSetEditable}
					<Button big onclick={showRenameEmoteDialog}>
						{#snippet icon()}
							<NotePencil />
						{/snippet}
						{$t("dialogs.emote_context_menu.edit_emote_alias")}
					</Button>
					<Button big onclick={showRemoveEmoteDialog}>
						{#snippet icon()}
							<Trash />
						{/snippet}
						{$t("dialogs.emote_context_menu.remove_emote")}
					</Button>
				{/if}
			{/if}
			<Button big href="/emotes/{data.id}" target="_blank" onclick={hide}>
				{#snippet icon()}
					<ArrowSquareOut />
				{/snippet}
				{$t("dialogs.emote_context_menu.open_in_new_tab")}
			</Button>
			<Button big onclick={copyLink}>
				{#snippet icon()}
					<Clipboard />
				{/snippet}
				{$t("dialogs.emote_context_menu.copy_emote_link")}
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
