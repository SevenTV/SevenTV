<script lang="ts">
	import { createEventDispatcher, onMount } from "svelte";
	import { fade } from "svelte/transition";
	import mouseTrap from "$/lib/mouseTrap";
	import { X } from "phosphor-svelte";
	import Button from "../button.svelte";

	const dispatch = createEventDispatcher();

	let dialog: HTMLDialogElement;

	export let width: number = 25;
	export let showClose: boolean = true;

	onMount(() => {
		dialog.showModal();
	});

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === "Escape") {
			dispatch("close");
		}
	}

	function onClose() {
		dispatch("close");
	}
</script>

<svelte:window on:keydown={handleKeyDown} />

<dialog
	bind:this={dialog}
	use:mouseTrap={onClose}
	aria-modal="true"
	transition:fade={{ duration: 100 }}
	style="--width-prop: {width}rem"
>
	<div use:mouseTrap={onClose}>
		<slot />
		{#if showClose}
			<Button on:click={() => dispatch("close")} style="position: absolute; top: 0.5rem; right: 0.5rem;">
				<X slot="icon" />
			</Button>
		{/if}
	</div>
</dialog>

<style lang="scss">
	dialog {
		margin: auto;
		border: none;
		padding: 0;
		background: none;

		max-width: 90vw;
		width: var(--width-prop);

		div {
			color: var(--text);
			font-weight: 500;

			width: 100%;
			border-radius: 0.5rem;
			background-color: var(--bg-dark);
		}

		&::backdrop {
			background-color: rgba(0, 0, 0, 0.5);
			backdrop-filter: blur(0.2rem);
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
