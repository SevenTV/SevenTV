<script lang="ts">
	import type { Snippet } from "svelte";

	let { count, children }: { count: number; children?: Snippet } = $props();

	let text = $derived(count > 99 ? "99+" : count.toString());
</script>

{#if children}
	{#if count > 0}
		<div class="badge-container">
			<span class="badge absolute">{text}</span>
			{@render children()}
		</div>
	{:else}
		{@render children()}
	{/if}
{:else if count > 0}
	<span class="badge">{text}</span>
{/if}

<style lang="scss">
	.badge-container {
		position: relative;
		display: inline-block;
	}

	.badge {
		display: flex;
		align-items: center;
		justify-content: center;

		color: var(--text);
		font-size: 0.7rem;
		font-weight: 500;
		background-color: var(--danger);

		width: calc(1.4rem - 3px);
		height: calc(1.4rem - 3px);
		border-radius: 50%;

		&.absolute {
			position: absolute;
			top: -0.4rem;
			right: -0.5rem;

			width: 1.4rem;
			height: 1.4rem;
			border: 3px solid var(--bg-dark);
		}
	}
</style>
