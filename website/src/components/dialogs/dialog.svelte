<script context="module" lang="ts">
	export enum DialogMode {
		Hidden = 0,
		Shown = 1,
		ShownWithoutClose = 2,
	};
</script>

<script lang="ts">
	import { fade } from "svelte/transition";
	import mouseTrap from "$/lib/mouseTrap";
	import { X } from "phosphor-svelte";
	import Button from "../button.svelte";
	import { browser } from "$app/environment";

	let dialog: HTMLDialogElement;

	export let mode: DialogMode = DialogMode.Hidden;
	export let width: number = 25;

	function close() {
		if (mode === DialogMode.Shown) {
			mode = DialogMode.Hidden;
		}
	}

	$: if (mode) {
		dialog?.showModal();

		// Focus and immediately blur to prevent visible autofocus
		// https://stackoverflow.com/a/76827288/10772729
		dialog?.focus();
		dialog?.blur();
	} else {
		dialog?.close();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (mode && event.key === "Escape") {
			close();
			if (mode === DialogMode.ShownWithoutClose) {
				event.preventDefault();
			}
		}
	}
</script>

<svelte:window on:keydown={handleKeyDown} />

<dialog
	bind:this={dialog}
	use:mouseTrap={close}
	aria-modal="true"
	transition:fade={{ duration: 100 }}
	style="width: {width}rem"
>
	{#if mode === DialogMode.Shown}
		<Button
			on:click={close}
			style="position: absolute; top: 0.5rem; right: 0.5rem;"
		>
			<X slot="icon" />
		</Button>
	{/if}
	<div class="trap" use:mouseTrap={close}>
		<slot />
	</div>
</dialog>

<style lang="scss">
	dialog {
		margin: auto;
		border: none;
		padding: 0;
		background: none;
		border-radius: 0.5rem;

		max-width: 90vw;
		overflow: auto;

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
