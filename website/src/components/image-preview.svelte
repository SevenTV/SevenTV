<script lang="ts">
	export let size = 64;
	export let src: string | null = null;
</script>

<div class="preview">
	{#if src}
		<img width={size} height={size} src={src} alt="preview" />
	{:else}
		<div class="image" style="height: {size}px; width: {size}px"></div>
	{/if}
	<span class="size-text">{size}x{size}</span>
</div>

<style lang="scss">
	@keyframes loading {
		0% {
			background-color: var(--primary);
		}
		50% {
			background-color: var(--primary-active);
		}
		100% {
			background-color: var(--primary);
		}
	}

	.preview {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.image {
		background-color: var(--primary);
		animation: loading 1s infinite;
	}

	.size-text {
		color: var(--text-lighter);
		font-size: 0.75rem;
	}
</style>
