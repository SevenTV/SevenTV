<script lang="ts">
	export let count: number;

	$: text = count > 99 ? "99+" : count.toString();
</script>

<div class="badge-container">
	<span class="badge">{text}</span>
	<slot />
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
