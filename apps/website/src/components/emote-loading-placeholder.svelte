<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = {
		index?: number;
		size?: number;
		children?: Snippet;
	} & HTMLAttributes<HTMLDivElement>;

	let { index = 0, size, children, ...restProps }: Props = $props();
</script>

<div
	class="placeholder loading-animation"
	style:animation-delay="{-index * 10}ms"
	style:width="{size}px"
	style:height="{size}px"
	{...restProps}
>
	{@render children?.()}
</div>

<style lang="scss">
	.placeholder {
		width: 100%;
		max-width: 10rem;
		aspect-ratio: 1 / 1;

		border: 1px solid transparent;
		border-radius: 0.25rem;

		background-color: var(--preview);
	}
</style>
