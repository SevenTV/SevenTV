<script lang="ts">
	let { tags = [] }: { tags?: string[] } = $props();
</script>

<div class="tags">
	{#each tags as tag}
		<span class="tag">{tag}</span>
	{/each}
</div>

<style lang="scss">
	.tags {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
		justify-content: center;
	}

	.tag {
		padding: 0.5rem 1rem;

		color: var(--text);
		font-size: 0.875rem;
		background-color: var(--secondary);
		border-radius: 0.5rem;
	}
</style>
