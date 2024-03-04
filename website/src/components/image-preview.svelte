<script lang="ts">
	export let size = 64;
	export let src: string | null = null;
</script>

<div class="preview">
	{#if src}
		<img width={size} height={size} {src} alt="preview" />
	{:else}
		<div class="image" style="height: {size}px; width: {size}px"></div>
	{/if}
	<span class="size-text">{size}x{size}</span>
</div>

<style lang="scss">
	@keyframes loading {
		0% {
			opacity: 0.5;
		}
		50% {
			opacity: 1;
		}
		100% {
			opacity: 0.5;
		}
	}

	.preview {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.image {
		background-color: var(--preview);
		animation: loading 1s infinite;
	}

	.size-text {
		color: var(--text-light);
		font-size: 0.75rem;
	}
</style>
