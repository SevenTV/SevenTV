<script lang="ts" context="module">
	let dropDownIndex = 0;
</script>

<script lang="ts">
	import mouseTrap from "$/lib/mouseTrap";
	import { fade } from "svelte/transition";

	export let hideOnMobile = false;
	export let hideOnDesktop = false;

	export let align: "left" | "right" = "right";

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

<div
	class="dropdown"
	use:mouseTrap={close}
	class:hide-on-mobile={hideOnMobile}
	class:hide-on-desktop={hideOnDesktop}
>
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<!-- This is just a wrapper element to catch the underlying click event -->
	<div
		class="input-wrapper"
		on:click|preventDefault={toggle}
		aria-expanded={expanded}
		aria-controls="dropdown-list-{index}"
		{...$$restProps}
	>
		<slot />
	</div>
	{#if expanded}
		<div class="dropped" id="dropdown-list-{index}" transition:fade={{ duration: 100 }} style={align === "left" ? "left: 0" : "right: 0"}>
			<slot name="dropdown" />
		</div>
	{/if}
</div>

<style lang="scss">
	.dropdown {
		position: relative;

		.input-wrapper {
			display: flex;
			align-items: center;
			gap: 0.5rem;

			cursor: pointer;
		}
	}

	.dropped {
		z-index: 1;

		position: absolute;
		top: 100%;
		margin: 0;
		margin-top: 0.25rem;
		padding: 0;
		border: var(--border-active) 1px solid;
		border-radius: 0.5rem;

		background-color: var(--bg-medium);
		box-shadow: 4px 4px 8px rgba(0, 0, 0, 0.1);
	}
</style>
