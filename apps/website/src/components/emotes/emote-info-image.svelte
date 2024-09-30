<script lang="ts">
	import type { Image } from "$/gql/graphql";

	// This should contain different file types of the same image
	export let images: (Image | undefined)[];

	$: first = images.find((i) => i);

	let loading: boolean = true;
</script>

<div class="image">
	{#if first}
		<picture>
			{#each images as image}
				{#if image}
					<source srcset={image.url} width={image.width} height={image.height} type={image.mime} />
				{/if}
			{/each}
			<img
				src={first.url}
				alt="Preview"
				style:width="{first.width}px"
				style:height="{first.height}px"
				class:loading
				on:load={() => (loading = false)}
			/>
		</picture>
		<span class="size-text">{first.width}x{first.height}</span>
	{/if}
</div>

<style lang="scss">
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

	.image {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.loading {
		background-color: var(--preview);
		animation: loading 1s infinite;
	}

	.size-text {
		color: var(--text-light);
		font-size: 0.75rem;
	}
</style>
