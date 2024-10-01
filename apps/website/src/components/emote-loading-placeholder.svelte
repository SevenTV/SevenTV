<script lang="ts">
	export let index = 0;
</script>

<div class="placeholder loading-animation" style:animation-delay="{-index * 10}ms"></div>

<style lang="scss">
	.placeholder {
		width: 100%;
		max-width: 10rem;
		aspect-ratio: 1 / 1;

		border: 1px solid transparent;
		border-radius: 0.25rem;

		background-color: var(--preview);
	}
</style>
