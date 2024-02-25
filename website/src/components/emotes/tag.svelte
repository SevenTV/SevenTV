<script lang="ts">
	export let name: string;
</script>

<span>{name}</span>

<style lang="scss">
	span {
		padding: 0.5rem 1rem;

		color: var(--text);
		font-size: 0.875rem;
		background-color: var(--secondary);
		border-radius: 0.5rem;
	}
</style>
