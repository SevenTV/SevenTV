<script lang="ts">
	import Button from "./button.svelte";

	export let user = "channelName";
	export let index = 0;
	export let big: boolean = false;
</script>

<Button href="/user/{user}" {big}>
	<div class="image" style="animation-delay: {-index * 10}ms" slot="icon"></div>
	<span class="user">{user}</span>
</Button>

<style lang="scss">
	.image {
		flex-shrink: 0;
		width: 2rem;
		height: 2rem;
		border-radius: 50%;

		background-color: var(--preview);
		animation: loading 1s infinite;
	}

	@keyframes loading {
		0% {
			opacity: 0.5;
		}
		50% {
			opacity: 1;
		}
		100% {
			opacity: 0.5;
		}
	}

	.user {
		flex-grow: 1;
		overflow: hidden;
		text-overflow: ellipsis;

		font-size: 0.875rem;
		font-weight: 600;
	}
</style>
