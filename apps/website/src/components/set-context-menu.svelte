<script lang="ts">
	import { ArrowSquareOut, Clipboard, FolderSimple } from "phosphor-svelte";
	import AddEmoteDialog from "./dialogs/add-emote-dialog.svelte";
	import Button from "./input/button.svelte";
	import mouseTrap from "$/lib/mouseTrap";
	import { page } from "$app/stores";
	import { fade } from "svelte/transition";
	import { user } from "$/lib/auth";
	import type { DialogMode } from "./dialogs/dialog.svelte";

	interface Props {
		setId: string;
		position?: { x: number; y: number };
	}

	let { setId, position = $bindable() }: Props = $props();

	function hide() {
		position = undefined;
	}

	let addEmoteDialogMode = $state<DialogMode>("hidden");

	$effect(() => {
		if (addEmoteDialogMode === "hidden") {
			hide();
		}
	});

	function showAddEmoteDialog() {
		addEmoteDialogMode = "shown";
	}

	function copyLink() {
		navigator.clipboard.writeText(new URL(`/emote-sets/${setId}`, $page.url).href);
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
				<AddEmoteDialog bind:mode={addEmoteDialogMode} setId={setId} />
				<Button big onclick={showAddEmoteDialog}>
					{#snippet icon()}
						<FolderSimple />
					{/snippet}
					Add Emote to...
				</Button>
			{/if}
			<Button big href="/emote-sets/{setId}" target="_blank" onclick={hide}>
				{#snippet icon()}
					<ArrowSquareOut />
				{/snippet}
				Go To Set
			</Button>
			<Button big onclick={copyLink}>
				{#snippet icon()}
					<Clipboard />
				{/snippet}
				Copy Set Link
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
