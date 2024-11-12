<script lang="ts">
	import type { Snippet } from "svelte";

	let { count, children }: { count: number; children: Snippet } = $props();

	let text = $derived(count > 99 ? "99+" : count.toString());
</script>

<div class="badge-container" {children}>
	{#if count > 0}
		<span class="badge">{text}</span>
	{/if}
</div>

<style lang="scss">
	.badge-container {
		position: relative;
		display: inline-block;
	}

	.badge {
		position: absolute;
		top: -0.75rem;
		right: -0.8rem;

		display: flex;
		align-items: center;
		justify-content: center;

		width: 1.4rem;
		height: 1.4rem;
		border-radius: 50%;
		background-color: var(--danger);
		border: 3px solid var(--bg-dark);
		font-size: 0.7rem;
		font-weight: 500;
	}
</style>
