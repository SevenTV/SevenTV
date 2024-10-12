<script lang="ts" generics="C extends ComponentType">
	// https://stackoverflow.com/a/72532661/10772729
	// eslint doesn't seem to understand this syntax

	import mouseTrap from "$/lib/mouseTrap";
	import { CaretDown } from "phosphor-svelte";
	import { fade } from "svelte/transition";
	import Button from "./button.svelte";
	import type { ComponentType } from "svelte"; // eslint-disable-line @typescript-eslint/no-unused-vars
	import { t } from "svelte-i18n";

	type Option = {
		value: string;
		label: string;
		icon?: C; // eslint-disable-line no-undef
	};

	export let options: Option[];
	export let selected: string | null = options[0]?.value ?? null;
	export let grow: boolean = false;

	$: selectedLabel = options.find((o) => o.value === selected);

	let expanded = false;

	function toggle() {
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

<div
	use:mouseTrap
	on:outsideClick={close}
	class="select"
	class:grow
	class:expanded
	{...$$restProps}
>
	<select bind:value={selected} on:click={toggle} on:keypress={toggle}>
		{#each options as option}
			<option value={option.value}>
				{option.value}
			</option>
		{/each}
	</select>
	<Button secondary tabindex="-1" on:click={toggle}>
		{#if selectedLabel}
			{#if selectedLabel.icon}
				<svelte:component this={selectedLabel.icon} />
			{/if}
			{selectedLabel.label}
		{:else}
			{$t("labels.select")}
		{/if}
		<CaretDown slot="icon-right" size={1 * 16} />
	</Button>
	{#if expanded}
		<div class="dropped" transition:fade={{ duration: 100 }}>
			{#each options as option}
				<Button on:click={() => select(option.value)}>
					{#if option.icon}
						<svelte:component this={option.icon} />
					{/if}
					{option.label}
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

		&:focus-visible + :global(.button) {
			border-color: var(--primary);
		}
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
		}

		&.expanded {
			& > :global(.button) {
				border-color: var(--border-active);
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
		border: var(--border-active) 1px solid;
		border-top: none;
		border-radius: 0.5rem;

		background-color: var(--bg-medium);
		box-shadow: 4px 4px 8px rgba(0, 0, 0, 0.1);

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
