<script lang="ts">
	export let size = 64;
</script>

<div class="preview">
	<div class="image" style="height: {size}px; width: {size}px"></div>
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
