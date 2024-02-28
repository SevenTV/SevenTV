<script lang="ts">
	import mouseTrap from "$/lib/mouseTrap";
	import { CaretDown } from "phosphor-svelte";
	import { fade } from "svelte/transition";
	import Button from "./button.svelte";

	export let options: string[];
	export let selected: string | null = options[0] ?? null;
	export let grow: boolean = false;

	let expanded = false;

	function toggle() {
		console.log("toggle");
		expanded = !expanded;
	}

	function close() {
		expanded = false;
	}

	function select(option: string) {
		selected = option;
		expanded = false;
	}
</script>

<div use:mouseTrap={close} class="select" class:grow class:expanded>
	<Button primary tabindex="-1" on:click={toggle}>
		{selected ?? "Select"}
		<CaretDown slot="icon-right" size="1rem" />
	</Button>
	<select bind:value={selected} on:click={toggle} on:keypress={toggle}>
		{#each options as option}
			<option value={option}>
				{option}
			</option>
		{/each}
	</select>
	{#if expanded}
		<div class="dropped" transition:fade={{ duration: 100 }}>
			{#each options as option}
				<Button primary={selected === option} on:click={() => select(option)}>
					{option}
				</Button>
			{/each}
		</div>
	{/if}
</div>

<style lang="scss">
	select {
		-webkit-appearance: none;
		-moz-appearance: none;
		appearance: none;
		outline: none;
		margin: 0;
		padding: 0;
		border: none;
		width: 0;

		display: inline;
		clip: rect(0 0 0 0);
		clip-path: inset(50%);
		height: 1px;
		overflow: hidden;
		position: absolute;
		white-space: nowrap;
		width: 1px;
	}

	.select {
		position: relative;

		&.grow {
			width: 100%;
			flex-grow: 1;
		}

		& > :global(.button) {
			width: 100%;

			justify-content: space-between;
			border: transparent 1px solid;

			&:focus-within {
				border-color: var(--secondary);
			}
		}

		&.expanded {
			& > :global(.button) {
				border-color: var(--border);
				border-bottom-left-radius: 0;
				border-bottom-right-radius: 0;
			}

			& > .dropped {
				border-top-left-radius: 0;
				border-top-right-radius: 0;
			}
		}
	}

	.dropped {
		z-index: 1;

		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin: 0;
		padding: 0;
		border: var(--border) 1px solid;
		border-top: none;
		border-radius: 0.5rem;

		background-color: var(--bg-medium);
		box-shadow: 4px 4px 0px rgba(0, 0, 0, 0.25);

		& > :global(.button) {
			border-radius: 0;
			width: 100%;
			padding: 0.5rem 1rem;

			&:last-child {
				border-bottom-left-radius: 0.5rem;
				border-bottom-right-radius: 0.5rem;
			}
		}
	}
</style>
