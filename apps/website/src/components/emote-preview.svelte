<script lang="ts" context="module">
	// From least supported to best supported
	const FORMAT_SORT_ORDER = [
		"image/avif",
		"image/webp",
		"image/gif",
		"image/png",
	];

	export function formatSortIndex(image: Image): number {
		let index = FORMAT_SORT_ORDER.indexOf(image.mime);

		// Static images first
		if (image.frameCount > 1) {
			index += FORMAT_SORT_ORDER.length;
		}

		return index;
	}
</script>

<script lang="ts">
	import type { Emote, Image } from "$/gql/graphql";
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

	let flags: string[] = [];

	let picture: HTMLPictureElement;

	// 60% * 10rem is the max size for the image
	$: size = picture?.clientWidth ?? (0.6 * 10 * 16);

	// This function prepares the variants for the <picture> element by grouping them by format, sorting them by scale and generating the required media and srcSet tags.
	// It also returns the best supported variant for use in the fallback <img> element which is the smallest GIF or PNG.
	function prepareVariants(images: Image[]): {
		bestSupported: Image | null;
		variants: { type: string; srcSet: string; media: string }[];
	} {
		if (!images) return { bestSupported: null, variants: [] };

		const animated = images.some((i) => i.frameCount > 1);

		images.sort((a, b) => a.scale - b.scale);

		const grouped: {
			type: string;
			srcSet: string;
			media: string;
			images: Image[];
		}[] = Object.values(
			images.reduce(
				(res, i) => {
					const index = formatSortIndex(i);
					if (!res[index]) {
						// Always true
						let media = "(min-width: 0px)";
						if (i.frameCount === 1 && animated) {
							media += " and (prefers-reduced-motion: reduce)";
						}
						res[index] = { type: i.mime, srcSet: "", media, images: [] };
					}
					res[index].images.push(i);
					return res;
				},
				{} as {
					[key: number]: {
						type: string;
						srcSet: string;
						media: string;
						images: Image[];
					};
				},
			),
		);

		const bestSupported =
			grouped[FORMAT_SORT_ORDER.indexOf("image/gif")]?.images[0] ??
			grouped[FORMAT_SORT_ORDER.indexOf("image/png")]?.images[0] ??
			null;

		// add srcset
		for (let i = 0; i < grouped.length; i++) {
			const srcSet = grouped[i].images
				.map((i) => `${i.url} ${i.width}w`)
				.join(", ");
			grouped[i].srcSet = srcSet;
		}

		return {
			bestSupported,
			variants: grouped,
		};
	}

	$: preparedVariants = prepareVariants(data.images);

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
	class:emote-only={emoteOnly}
	style:border-color={selected ? "var(--primary)" : `${highlight}80`}
	style:background-color="var(--bg-{bg})"
	title={data.defaultName}
	on:click={onClick}
	{...$$restProps}
>
	<picture bind:this={picture}>
		{#each preparedVariants.variants as variant}
			<source
				type={variant.type}
				srcset={variant.srcSet}
				sizes="{size}px"
				media={variant.media}
			/>
		{/each}
		<img
			class="image"
			src="{preparedVariants.bestSupported?.url}"
			style="animation-delay: {-index * 10}ms"
			on:load={() => (loading = false)}
			alt={data.defaultName}
			class:loading
		/>
	</picture>
	{#if !emoteOnly}
		<span class="name">{data.defaultName}</span>
		{#if data.owner?.mainConnection?.platformDisplayName}
			<span class="user">{data.owner.mainConnection.platformDisplayName}</span>
		{/if}
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

		&.emote-only picture {
			max-width: 100%;
			max-height: 100%;
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

	picture {
		flex-grow: 1;
		margin-bottom: 0.5rem;
		line-height: 0;

		width: 100%;
		max-width: 60%;
		max-height: 50%;
	}

	.image {
		object-fit: contain;

		width: 100%;
		height: 100%;
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
