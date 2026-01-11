<script lang="ts">
	import { ArrowSquareOut, Clipboard, FolderSimple } from "phosphor-svelte";
	import AddEmoteDialog from "./dialogs/add-emote-dialog.svelte";
	import Button from "./input/button.svelte";
	import mouseTrap from "$/lib/mouseTrap";
	import type { DialogMode } from "./dialogs/dialog.svelte";
	import type { Emote } from "$/gql/graphql";
	import EmoteUseButton from "./emote-use-button.svelte";
	import { page } from "$app/stores";
	import { fade } from "svelte/transition";
	import { user } from "$/lib/auth";

	interface Props {
		data: Emote;
		position?: { x: number; y: number };
	}

	let { data, position = $bindable() }: Props = $props();

	function hide() {
		position = undefined;
	}

	let addEmoteDialogMode = $state<DialogMode>("hidden");

	$effect(() => {
		if (addEmoteDialogMode === "hidden") hide();
	});

	function showAddEmoteDialog() {
		addEmoteDialogMode = "shown";
	}

	function copyLink() {
		const url = new URL(`/emotes/${data.id}`, $page.url).href;
		navigator.clipboard.writeText(url);
		hide();
		showToast("Emote link copied!");
	}

	function copyCdn2x() {
 	   if (!data || !data.id) return;

 	  const emoteId = data.id;
 	   const url = `https://cdn.7tv.app/emote/${emoteId}/2x.avif`;

    	navigator.clipboard.writeText(url)
        	.then(() => showToast("2x CDN link copied!"))
        	.catch(() => showToast("Failed to copy CDN link!"));

    	hide();
	}


	function showToast(message: string) {
		const toast = document.createElement("div");
		toast.textContent = message;
		Object.assign(toast.style, {
			position: "fixed",
			bottom: "2rem",
			left: "50%",
			transform: "translateX(-50%)",
			background: "var(--bg-dark)",
			color: "white",
			padding: "0.5rem 1rem",
			borderRadius: "0.25rem",
			zIndex: "9999",
			fontSize: "0.875rem",
		});
		document.body.appendChild(toast);
		setTimeout(() => toast.remove(), 2000);
	}

	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
	}
</script>

{#if position}
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
				<Button big on:click={showAddEmoteDialog}>
					{#snippet icon()}<FolderSimple />{/snippet}
					Add Emote to...
				</Button>
			{/if}

			<Button big href="/emotes/{data.id}" target="_blank" on:click={hide}>
				{#snippet icon()}<ArrowSquareOut />{/snippet}
				Open in New Tab
			</Button>

			<Button big on:click={copyLink}>
				{#snippet icon()}<Clipboard />{/snippet}
				Copy Emote Link
			</Button>

			<Button big on:click={copyCdn2x}>
    			{#snippet icon()}<Clipboard />{/snippet}
			    Copy CDN Link (2x)
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
			padding: 0.5rem;
			z-index: 150;
		}
	}
</style>
