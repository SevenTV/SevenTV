<script lang="ts">
	export let name = "emoteName";
	export let index = 0;
	export let bg: "medium" | "light" = "medium";
	export let emoteOnly = false;
</script>

<a href="/emotes/{name}" class="emote" class:emote-only={emoteOnly} style:background-color="var(--bg-{bg})" title={name}>
	<div class="image" style="animation-delay: {index * 10}ms"></div>
	{#if !emoteOnly}
		<span class="name">{name}</span>
		<span class="user">username</span>
	{/if}
</a>

<style lang="scss">
	.emote {
		color: var(--text);
		text-decoration: none;

		width: 100%;
		aspect-ratio: 1 / 1;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;

		border: 1px solid transparent;
		border-radius: 0.25rem;
		cursor: pointer;

		&:hover,
		&:focus-visible {
			border-color: var(--border-active);
		}

		&.emote-only > .image {
			width: 100%;
			height: 100%;
			margin: 0;
		}
	}

	@keyframes loading {
		0% {
			opacity: 0.5;
		}
		50% {
			opacity: 1.0;
		}
		100% {
			opacity: 0.5;
		}
	}

	.image {
		width: 50%;
		height: 50%;
		margin-bottom: 0.5rem;

		background-color: var(--preview);
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
