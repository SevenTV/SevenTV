<script lang="ts">
	export let user = "channelName";
	export let index = 0;
	export let bg: "medium" | "light" = "medium";
</script>

<a href="/user/{user}" class="channel" style:background-color="var(--bg-{bg})">
	<div class="image" style="animation-delay: {index * 10}ms"></div>
	<span class="user">{user}</span>
</a>

<style lang="scss">
	.channel {
		color: var(--text);
		text-decoration: none;

		padding: 0.5rem 1rem;

		display: flex;
		align-items: center;
		gap: 0.62rem;

		border: 1px solid transparent;
		border-radius: 0.25rem;

		&:hover,
		&:focus-visible {
			border-color: var(--border);
		}
	}

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

	.image {
		flex-shrink: 0;
		width: 2.5rem;
		height: 2.5rem;
		border-radius: 50%;

		background-color: var(--primary);
		animation: loading 1s infinite;
	}

	.user {
		flex-grow: 1;
		overflow: hidden;
		text-overflow: ellipsis;

		font-size: 0.875rem;
		font-weight: 600;
	}
</style>
