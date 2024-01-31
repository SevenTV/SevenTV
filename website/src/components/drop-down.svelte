<script context="module">
	let dropDownIndex = 0;
</script>

<script lang="ts">
	import mouseTrap from "$/lib/mouseTrap";
	import { fade } from "svelte/transition";

	export let button: boolean = false;

	let index = dropDownIndex;
	dropDownIndex += 1;

	let expanded = false;

	function toggle() {
		expanded = !expanded;
	}

	function close() {
		expanded = false;
	}
</script>

<button
	on:click={toggle}
	aria-expanded={expanded}
	aria-controls="dropdown-list-{index}"
	class:button
	use:mouseTrap={close}
>
	<slot />
	{#if expanded}
		<div class="dropped" id="dropdown-list-{index}" transition:fade={{ duration: 100 }}>
			<slot name="dropdown" />
		</div>
	{/if}
</button>

<style lang="scss">
	button {
		position: relative;
		display: flex;
		align-items: center;
		gap: 0.5rem;

		.dropped {
			z-index: 1;

			position: absolute;
			top: 100%;
			right: 0;
			margin: 0;
			padding: 0;
			border: var(--border) 1px solid;
			border-radius: 0.5rem;

			background-color: var(--bg-medium);
			box-shadow: 4px 4px 0px rgba(0, 0, 0, 0.25);
		}
	}
</style>
