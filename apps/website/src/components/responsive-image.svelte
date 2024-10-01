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
	import type { Image } from "$/gql/graphql";

	export let images: Image[];
	export let alt = "";
	export let index = 0;

	let loading: boolean = true;

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

	$: preparedVariants = prepareVariants(images);
</script>

<picture bind:this={picture} {...$$restProps}>
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
		loading="lazy"
		style="animation-delay: {-index * 10}ms"
		on:load={() => (loading = false)}
		alt={alt}
		class:loading
	/>
</picture>

<style lang="scss">
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
</style>
