<script lang="ts">
	export let size = 64;
</script>

<div class="image" style="height: {size}px; width: {size}px"></div>

<style lang="scss">
	@keyframes loading {
		0% {
			background-color: var(--secondary);
		}
		50% {
			background-color: var(--secondary-active);
		}
		100% {
			background-color: var(--secondary);
		}
	}

	.image {
		margin-bottom: 0.5rem;

		background-color: var(--secondary);
		animation: loading 1s infinite;
	}
</style>
