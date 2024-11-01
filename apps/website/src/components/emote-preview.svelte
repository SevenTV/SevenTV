<script lang="ts">
	import type { Emote } from "$/gql/graphql";
	import Flags, { emoteToFlags, determineHighlightColor } from "./flags.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import ResponsiveImage from "./responsive-image.svelte";

	export let data: Emote;
	export let index = 0;
	export let bg: "medium" | "light" = "medium";
	export let emoteOnly = false;
	export let selectionMode = false;
	export let selected = false;
	export let ignoredFlagsForHighlight: string[] = [];

	$: flags = emoteToFlags(data);

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
	data-sveltekit-preload-data="tap"
	class="emote"
	class:emote-only={emoteOnly}
	class:selected
	style={highlight
		? `--highlight: ${highlight}80; --highlight-active: ${highlight};`
		: "--highlight: transparent; --highlight-active: var(--border-active);"}
	style:background-color="var(--bg-{bg})"
	title={data.defaultName}
	on:click={onClick}
	{...$$restProps}
>
	<ResponsiveImage images={data.images} alt={data.defaultName} {index} />
	{#if !emoteOnly}
		<span class="name">{data.defaultName}</span>
		{#if data.owner?.mainConnection?.platformDisplayName}
			<span class="user" style:color={data.owner.highestRoleColor?.hex}
				>{data.owner.mainConnection.platformDisplayName}</span
			>
		{/if}
	{/if}
	{#if selectionMode || flags.length > 0}
		<div class="flags">
			{#if selectionMode}
				<Checkbox bind:value={selected} />
			{/if}
			{#if !emoteOnly && flags.length > 0}
				<Flags {flags} iconOnly style="flex-direction: column; gap: 0.4rem;" />
			{/if}
		</div>
	{/if}
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

		border-color: var(--highlight);

		&.selected {
			border-color: var(--primary);
		}

		&:hover,
		&:focus-visible {
			border-color: var(--highlight-active);
		}

		& > :global(picture) {
			flex-grow: 1;
			margin-bottom: 0.5rem;
			line-height: 0;

			width: 100%;
			max-width: 60%;
			max-height: 50%;
		}

		& > :global(picture > img) {
			object-fit: contain;

			width: 100%;
			height: 100%;
		}

		&.emote-only > :global(picture) {
			max-width: 100%;
			max-height: 100%;
			margin: 0;
		}
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
