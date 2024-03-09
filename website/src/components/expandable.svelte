<script lang="ts">
	import { CaretDown } from "phosphor-svelte";

	export let title: string;

	let expanded = true;
</script>

<div class="expandable" {...$$restProps}>
	<button class="header" on:click={() => (expanded = !expanded)} class:expanded>
		{title}
		<div class="icon">
			<CaretDown size="1rem" />
		</div>
	</button>
	{#if expanded}
		<slot />
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
