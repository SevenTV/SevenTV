<script lang="ts">
	export let user = "channelName";
	export let index = 0;
	export let bg: "medium" | "light" = "medium";
</script>

<a href="/channel/{user}" class="channel" style:background-color="var(--bg-{bg})">
	<div class="image" style="animation-delay: {index * 10}ms"></div>
	<span class="user">{user}</span>
</a>

<style lang="scss">
	.channel {
		color: var(--text);
		text-decoration: none;

		width: 10rem;
		height: 3.5rem;

		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 0.45rem;

		border: 1px solid transparent;
		border-radius: 0.25rem;
		cursor: pointer;

		&:hover,
		&:focus-visible {
			border-color: var(--border);
		}
	}

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
		width: 2.5rem;
		height: 2.5rem;
		border-radius: 50%;

		background-color: var(--secondary);
		animation: loading 1s infinite;
	}

	.user {
		font-weight: 500;
		margin-left: 0.5rem;
	}
</style>
