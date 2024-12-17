<script lang="ts" module>
	// From least supported to best supported
	const FORMAT_SORT_ORDER = ["image/avif", "image/webp", "image/gif", "image/png"];

	export function formatSortIndex(image: Image, splitStatic: boolean): number {
		let index = FORMAT_SORT_ORDER.indexOf(image.mime);

		// Static images first
		if (splitStatic && image.frameCount > 1) {
			index += FORMAT_SORT_ORDER.length;
		}

		return index;
	}
</script>

<script lang="ts">
	import type { Image } from "$/gql/graphql";
	import { reducedMotion, type ReducedMotion } from "$/lib/layout";
	import { browser } from "$app/environment";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = {
		images: Image[];
		width?: number;
		height?: number;
		round?: boolean;
		borderColor?: string;
		index?: number;
		draggable?: boolean;
	} & HTMLAttributes<HTMLPictureElement>;

	let {
		images,
		width,
		height,
		round = false,
		borderColor,
		index = 0,
		draggable = false,
		...restProps
	}: Props = $props();

	// https://stackoverflow.com/a/23522755/10772729
	const isSafari = browser ? /^((?!chrome|android).)*safari/i.test(navigator.userAgent) : false;

	let loading: boolean = $state(true);

	let pictureWidth: number | undefined = $state();

	// This function prepares the variants for the <picture> element by grouping them by format, sorting them by scale and generating the required media and srcSet tags.
	// It also returns the best supported variant for use in the fallback <img> element which is the smallest GIF or PNG.
	function prepareVariants(
		images: Image[],
		reducedMotion: ReducedMotion,
	): {
		bestSupported: Image | null;
		variants: { type: string; srcSet: string; media: string }[];
	} {
		if (!images) return { bestSupported: null, variants: [] };

		const animated = images.some((i) => i.frameCount > 1);

		images = images.toSorted((a, b) => a.scale - b.scale);

		const grouped: {
			type: string;
			srcSet: string;
			media: string;
			images: Image[];
		}[] = Object.values(
			images
				// Apple fails to implement the picture element correctly
				// It doesn't fall back to a supported format if the format isn't supported (the whole point of the picture element)
				// Apple doesn't fully support animated AVIF images, so we have to manually filter them out here
				.filter((i) => !(isSafari && i.mime === "image/avif" && i.frameCount > 1))
				.filter((i) => {
					if (reducedMotion === "reduced-motion-enabled" && animated) {
						// Return only static images
						return i.frameCount === 1;
					} else if (reducedMotion === "reduced-motion-disabled" && animated) {
						// Return only animated images
						return i.frameCount > 1;
					} else {
						return true;
					}
				})
				.reduce(
					(res, i) => {
						const index = formatSortIndex(i, reducedMotion === "reduced-motion-system");
						if (!res[index]) {
							// Always true
							let media = "(min-width: 0px)";
							if (reducedMotion === "reduced-motion-system" && i.frameCount === 1 && animated) {
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
			const srcSet = grouped[i].images.map((i) => `${i.url} ${i.width}w`).join(", ");
			grouped[i].srcSet = srcSet;
		}

		return {
			bestSupported,
			variants: grouped,
		};
	}

	let preparedVariants = $derived(prepareVariants(images, $reducedMotion));
</script>

<picture
	bind:clientWidth={pictureWidth}
	style:width="{width}px"
	style:height="{height}px"
	{...restProps}
>
	{#each preparedVariants.variants as variant}
		<source
			type={variant.type}
			srcset={variant.srcSet}
			sizes="{pictureWidth}px"
			media={variant.media}
			{width}
			{height}
		/>
	{/each}
	<!-- svelte-ignore a11y_missing_attribute -->
	<!-- Adding an alt here makes it glitch around, idk why but I don't care enough to figure it out -->
	<img
		class="image"
		class:round
		class:border={borderColor}
		style:border-color={borderColor}
		src={preparedVariants.bestSupported?.url}
		loading="lazy"
		style:animation-delay="{-index * 10}ms"
		onload={() => (loading = false)}
		class:loading-animation={loading}
		{draggable}
		{width}
		{height}
	/>
</picture>

<style lang="scss">
	.loading-animation {
		background-color: var(--preview);
	}

	.image.round {
		border-radius: 50%;
	}

	.image.border {
		border: 2px solid;
	}
</style>
