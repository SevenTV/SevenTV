<script lang="ts">
	import type { Snippet } from "svelte";

	interface Props {
		value: number;
		min: number;
		max: number;
		step?: number;
		disabled?: boolean;
		children?: Snippet;
	}

	let { value = $bindable(), min, max, step = 1, disabled = false, children } = $props();
</script>

<label>
	{@render children?.()}
	<input type="range" bind:value {min} {max} {step} {disabled} />
</label>

<style lang="scss">
	input {
		display: block;

		padding-block: 0.25rem;
		margin-top: 0.25rem;
	}
</style>
