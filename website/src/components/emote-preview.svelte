<script lang="ts">
	export let name = "emoteName";
	export let index = 0;
	export let bg: "medium" | "light" = "medium";
</script>

<a href="/emotes/{name}" class="emote" style:background-color="var(--bg-{bg})">
	<div class="image" style="animation-delay: {index * 10}ms"></div>
	<span class="name">{name}</span>
	<span class="user">username</span>
</a>

<style lang="scss">
	.emote {
		color: var(--text);
		text-decoration: none;

		width: 10rem;
		height: 10rem;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;

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
		width: 5rem;
		height: 5rem;
		margin-bottom: 0.5rem;

		background-color: var(--primary);
		animation: loading 1s infinite;
	}

	.name {
		font-weight: 500;
		max-width: 90%;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.user {
		font-size: 0.75rem;
		font-weight: 500;
	}
</style>
