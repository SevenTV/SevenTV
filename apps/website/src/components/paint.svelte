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

	let backgroundImage = $derived(
		paint.data.layers[0] ? layerToBackgroundImage(paint.data.layers[0]) : undefined,
	);
	let backgroundColor = $derived(
		paint.data.layers[0] ? layerToBackgroundColor(paint.data.layers[0]) : undefined,
	);
	let filter = $derived(paint.data.shadows.map(shadowToFilter).join(" "));
</script>

<span
	class="paint"
	style:background-image={backgroundImage}
	style:background-color={backgroundColor}
	style:filter
	title="Paint: {paint.name}"
	{...restProps}
>
	{@render children()}
</span>

<style lang="scss">
	.paint {
		background-color: currentColor;

		-webkit-text-fill-color: transparent;
		background-clip: text;
		-webkit-background-clip: text;
		background-size: cover;
	}
</style>
