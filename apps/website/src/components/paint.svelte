<script lang="ts">
	import {
		PaintRadialGradientShape,
		type Paint,
		type PaintLayer,
		type PaintShadow,
	} from "$/gql/graphql";
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = {
		paint: Paint;
		children: Snippet;
	} & HTMLAttributes<HTMLSpanElement>;

	let { paint, children, ...restProps }: Props = $props();

	function layerToBackgroundImage(layer: PaintLayer) {
		switch (layer.ty.__typename) {
			case "PaintLayerTypeLinearGradient":
				const linearRepeating = layer.ty.repeating ? "repeating-" : "";
				const linearStops = layer.ty.stops
					.map((stop) => `${stop.color.hex} ${stop.at * 100}%`)
					.join(", ");

				return `${linearRepeating}linear-gradient(${layer.ty.angle}deg, ${linearStops})`;
			case "PaintLayerTypeRadialGradient":
				const radialRepeating = layer.ty.repeating ? "repeating-" : "";

				let shape;
				switch (layer.ty.shape) {
					case PaintRadialGradientShape.Circle:
						shape = "circle";
						break;
					case PaintRadialGradientShape.Ellipse:
						shape = "ellipse";
						break;
				}

				const radialStops = layer.ty.stops
					.map((stop) => `${stop.color.hex} ${stop.at * 100}%`)
					.join(", ");

				return `${radialRepeating}radial-gradient(${shape}, ${layer.ty.angle}deg, ${radialStops})`;
			case "PaintLayerTypeImage":
				// TODO: Always uses 1x image for now
				const isAnimated = layer.ty.images.some((img) => img.frameCount > 1);
				const oneX = layer.ty.images.find(
					(img) => img.scale === 1 && img.frameCount > 1 === isAnimated,
				);

				if (!oneX) {
					return undefined;
				}

				return `url(${oneX.url})`;
			default:
				return undefined;
		}
	}

	function layerToBackgroundColor(layer: PaintLayer) {
		if (layer.ty.__typename === "PaintLayerTypeSingleColor") {
			return layer.ty.color.hex;
		}

		return undefined;
	}

	function shadowToFilter(shadow: PaintShadow) {
		return `drop-shadow(${shadow.color.hex} ${shadow.offsetX}px ${shadow.offsetY}px ${shadow.blur}px)`;
	}

	let layers = $derived(
		paint.data.layers.map((l) => {
			return {
				opacity: l.opacity,
				image: layerToBackgroundImage(l),
				color: layerToBackgroundColor(l),
			};
		}),
	);
	let filter = $derived(paint.data.shadows.map(shadowToFilter).join(" "));
</script>

<div class="paint" title="Paint: {paint.name}" {...restProps}>
	{#each layers as { opacity, image, color }, i}
		<!-- Apply filters only to first layer -->
		<span
			class="layer"
			style:opacity
			style:background-image={image}
			style:background-color={color}
			style:filter={i === 0 ? filter : undefined}
		>
			{@render children()}
		</span>
	{/each}
</div>

<style lang="scss">
	.paint {
		display: grid;
		justify-items: start;
	}

	.layer {
		// Overlay all layers on top of each other
		grid-area: 1 / 1 / -1 / -1;

		background-color: currentColor;

		-webkit-text-fill-color: transparent;
		background-clip: text;
		-webkit-background-clip: text;
		background-size: cover;
	}
</style>
