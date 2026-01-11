<script lang="ts" generics="T">
	import type { Snippet } from "svelte";
	import { crossfade } from "svelte/transition";
	import { cubicInOut } from "svelte/easing";

	export type SegmentedControlOption<T> = {
		value: T;
		label?: string;
		icon?: Snippet;
	};

	let {
		value = $bindable(),
		options,
	}: { value: T; options: SegmentedControlOption<T>[] } = $props();

	const [send, receive] = crossfade({
		duration: 200,
		easing: cubicInOut,
	});

	function handleClick(option: T) {
		value = option;
	}
</script>

<div class="segmented-control">
	{#each options as option}
		<button
			type="button"
			class:active={value === option.value}
			onclick={() => handleClick(option.value)}
		>
			{#if option.icon}
				{@render option.icon()}
			{/if}
			{#if option.label}
				<span class="label">{option.label}</span>
			{/if}
			{#if value === option.value}
				<div
					class="slider"
					in:receive={{ key: "slider" }}
					out:send={{ key: "slider" }}
				></div>
			{/if}
		</button>
	{/each}
</div>

<style lang="scss">
	.segmented-control {
		display: flex;
		background-color: var(--secondary);
		padding: 3px;
		border-radius: 0.5rem;
		gap: 0.25rem;
	}

	button {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.375rem 0.75rem;
		border: none;
		background: transparent;
		color: var(--text-light);
		font-weight: 600;
		font-size: 0.875rem;
		cursor: pointer;
		border-radius: 0.25rem;
		outline: none;
		z-index: 1;
		transition: color 0.2s;

		&.active {
			color: var(--text);
		}

		&:hover:not(.active) {
			color: var(--text);
		}
	}

	.slider {
		position: absolute;
		inset: 0;
		background-color: var(--bg-light);
		border-radius: 0.25rem;
		z-index: -1;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
	}
</style>
