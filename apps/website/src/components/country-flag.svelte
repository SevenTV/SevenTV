<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";

	type Props = {
		code: string;
		name: string;
		width?: number;
		height?: number;
	} & HTMLAttributes<HTMLImageElement>;

	let { code, name, width, height, ...restProps }: Props = $props();
</script>

<img
	src="/flags/{code}.svg"
	alt={name}
	title={name}
	class="flag"
	width="{width}px"
	height="{height}px"
	{...restProps}
/>

<style lang="scss">
	.flag {
		border-radius: 0.25rem;
	}
</style>
