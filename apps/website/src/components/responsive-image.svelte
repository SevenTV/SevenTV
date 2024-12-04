<script lang="ts" module>
	// From least supported to best supported
	const FORMAT_SORT_ORDER = ["image/avif", "image/webp", "image/gif", "image/png"];

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
	import type { Image } from "$/gql/graphql";
	import { type HTMLAttributes } from "svelte/elements";

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

	let loading: boolean = $state(true);

	let pictureWidth: number | undefined = $state();

	// This function prepares the variants for the <picture> element by grouping them by format, sorting them by scale and generating the required media and srcSet tags.
	// It also returns the best supported variant for use in the fallback <img> element which is the smallest GIF or PNG.
	function prepareVariants(images: Image[]): {
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
			const srcSet = grouped[i].images.map((i) => `${i.url} ${i.width}w`).join(", ");
			grouped[i].srcSet = srcSet;
		}

		return {
			bestSupported,
			variants: grouped,
		};
	}

	let preparedVariants = $derived(prepareVariants(images));
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
