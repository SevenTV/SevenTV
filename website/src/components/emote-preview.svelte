<script lang="ts">
	import type { Emote } from "$/gql/graphql";
	import Flags, { determineHighlightColor } from "./flags.svelte";
	import Checkbox from "./input/checkbox.svelte";

	export let data: Emote;
	export let index = 0;
	export let bg: "medium" | "light" = "medium";
	export let emoteOnly = false;
	export let selectionMode = false;
	export let selected = false;
	export let ignoredFlagsForHighlight: string[] = [];

	let loading: boolean = true;

	let flags = ["active", "global", "trending", "overlaying"];

	$: imageUrl = data.images
		.sort((a, b) => b.size - a.size)
		.find((i) => data.flags.animated === i.frameCount > 1)?.url;

	$: highlight = determineHighlightColor(flags, ignoredFlagsForHighlight);

	$: if (!selectionMode) {
		selected = false;
	}

	function onClick(e: MouseEvent) {
		if (selectionMode) {
			selected = !selected;
			e.preventDefault();
		}
	}
</script>

<a
	href="/emotes/{data.id}"
	class="emote"
	style:border-color={selected ? "var(--primary)" : `${highlight}80`}
	style:background-color="var(--bg-{bg})"
	title={data.defaultName}
	on:click={onClick}
	{...$$restProps}
>
	<img
		class="image"
		class:loading
		src={imageUrl}
		alt={data.defaultName}
		loading="lazy"
		style="animation-delay: {-index * 10}ms"
		on:load={() => (loading = false)}
	/>
	{#if !emoteOnly}
		<span class="name">{data.defaultName}</span>
		<span class="user">{data.ownerId}</span>
	{/if}
	<div class="flags">
		{#if selectionMode}
			<Checkbox bind:value={selected} />
		{/if}
		{#if !emoteOnly}
			<Flags {flags} iconOnly style="flex-direction: column; gap: 0.4rem;" />
		{/if}
	</div>
</a>

<style lang="scss">
	.emote {
		position: relative;

		color: var(--text);
		text-decoration: none;

		width: 100%;
		max-width: 10rem;
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

	.loading {
		animation: loading 1s infinite;
		background-color: var(--preview);
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

	.image {
		flex-grow: 1;
		object-fit: contain;

		width: 100%;
		height: 100%;
		max-width: 50%;
		max-height: 50%;

		margin-bottom: 0.5rem;
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
		max-width: 90%;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.flags {
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;

		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		align-items: center;
	}
</style>
