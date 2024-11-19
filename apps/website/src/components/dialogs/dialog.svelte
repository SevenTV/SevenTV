<script lang="ts" module>
	export type DialogMode = "hidden" | "shown" | "shown-without-close";
</script>

<script lang="ts">
	import { fade } from "svelte/transition";
	import mouseTrap from "$/lib/mouseTrap";
	import { X } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { browser } from "$app/environment";
	import type { Snippet } from "svelte";

	let dialog = $state<HTMLDialogElement>();

	let {
		mode = $bindable("hidden"),
		width = 25,
		children,
	}: { mode?: DialogMode; width?: number; children: Snippet } = $props();

	function close() {
		if (mode === "shown") {
			mode = "hidden";
		}
	}

	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		mode; // trigger effect on mode change

		dialog?.showModal();

		// Blur to prevent initial visible autofocus
		if (browser && document.activeElement instanceof HTMLElement) {
			document.activeElement.blur();
		}
	});

	function handleKeyDown(event: KeyboardEvent) {
		if (mode && event.key === "Escape") {
			close();
			if (mode === "shown-without-close") {
				event.preventDefault();
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if mode !== "hidden"}
	<dialog
		bind:this={dialog}
		use:mouseTrap={close}
		aria-modal="true"
		transition:fade={{ duration: 100 }}
		style="width: {width}rem"
	>
		{#if mode === "shown"}
			<Button onclick={close} style="position: absolute; top: 0.5rem; right: 0.5rem;">
				{#snippet icon()}
					<X />
				{/snippet}
			</Button>
		{/if}
		<div use:mouseTrap={close} class="trap">
			{@render children()}
		</div>
	</dialog>
{/if}

<style lang="scss">
	dialog {
		margin: auto;
		border: none;
		padding: 0;
		background: none;
		border-radius: 0.5rem;

		max-width: 90vw;
		overflow: auto;
		color: var(--text);

		background-color: var(--bg-dark);

		&::backdrop {
			background-color: rgba(0, 0, 0, 0.8);
		}

		.trap {
			display: contents;
		}
	}

	@media screen and (max-width: 960px) {
		dialog {
			max-width: 100vw;
			width: 100vw;

			max-height: 100vh;
			max-height: 100dvh;
			height: 100vh;
			height: 100dvh;
		}
	}
</style>
