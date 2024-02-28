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

		width: var(--emote-preview-size);
		height: var(--emote-preview-size);

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
		width: calc(var(--emote-preview-size) / 2);
		height: calc(var(--emote-preview-size) / 2);
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
