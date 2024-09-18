<script lang="ts">
	export let name: string;
</script>

<span>{name}</span>

<style lang="scss">
	span {
		padding: 0.2rem 0.5rem;

		color: var(--staff);
		font-size: 0.75rem;
		font-weight: 500;
		background-color: var(--staff-bg);
		border-radius: 0.5rem;
	}
</style>
