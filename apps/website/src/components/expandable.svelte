<script lang="ts">
	import { isMobileLayout } from "$/lib/utils";
	import { CaretDown } from "phosphor-svelte";
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = {
		title: string;
		expanded?: boolean;
		children: Snippet;
	} & HTMLAttributes<HTMLDivElement>;

	let { title, expanded = !isMobileLayout(), children, ...restProps }: Props = $props();
</script>

<div class="expandable" {...restProps}>
	<button class="header" onclick={() => (expanded = !expanded)} class:expanded>
		{title}
		<div class="icon">
			<CaretDown size={1 * 16} />
		</div>
	</button>
	{#if expanded}
		{@render children()}
	{/if}
</div>

<style lang="scss">
	.expandable {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;

		color: var(--text-light);
		font-size: 0.875rem;
		font-weight: 500;

		& > .icon {
			color: var(--text);

			transition: transform 0.1s;
			transform: rotate(-90deg);
		}

		&.expanded > .icon {
			transform: rotate(0deg);
		}
	}
</style>
