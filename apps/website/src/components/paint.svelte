<script lang="ts">
	import {
		PaintRadialGradientShape,
		type Paint,
		type PaintLayer,
		type PaintShadow,
	} from "$/gql/graphql";
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import PaintDialog from "./dialogs/paint-dialog.svelte";
	import type { DialogMode } from "./dialogs/dialog.svelte";

	type Props = {
		paint: Paint;
		children: Snippet;
		enableDialog?: boolean;
	} & HTMLAttributes<HTMLSpanElement> &
		HTMLAttributes<HTMLButtonElement>;

	let { paint, children, enableDialog, ...restProps }: Props = $props();

	function layerToBackgroundImage(layer: PaintLayer) {
		switch (layer.ty.__typename) {
			case "PaintLayerTypeLinearGradient": {
				if (layer.ty.stops.length === 0) {
					return undefined;
				}

				const linearRepeating = layer.ty.repeating ? "repeating-" : "";
				const linearStops = layer.ty.stops
					.map((stop) => `${stop.color.hex} ${stop.at * 100}%`)
					.join(", ");

				return `${linearRepeating}linear-gradient(${layer.ty.angle}deg, ${linearStops})`;
			}
			case "PaintLayerTypeRadialGradient": {
				if (layer.ty.stops.length === 0) {
					return undefined;
				}

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

				return `${radialRepeating}radial-gradient(${shape}, ${radialStops})`;
			}
			case "PaintLayerTypeImage": {
				const isAnimated = layer.ty.images.some((img) => img.frameCount > 1);

				// TODO: Always uses 1x image for now
				const oneX = layer.ty.images.find(
					(img) => img.scale === 1 && img.frameCount > 1 === isAnimated,
				);

				if (!oneX) {
					return undefined;
				}

				return `url(${oneX.url})`;
			}
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
		paint.data.layers
			.map((l) => {
				const image = layerToBackgroundImage(l);
				const color = layerToBackgroundColor(l);

				if (!image && !color) {
					return undefined;
				}

				return {
					opacity: l.opacity,
					image,
					color,
				};
			})
			.filter((l) => l !== undefined),
	);
	let filter = $derived(
		paint.data.shadows.length > 0 ? paint.data.shadows.map(shadowToFilter).join(" ") : undefined,
	);

	let dialogMode: DialogMode = $state("hidden");

	function showDialog() {
		dialogMode = "shown";
	}
</script>

<PaintDialog {paint} bind:mode={dialogMode} />

{#snippet content()}
	{#if layers.length === 0}
		<!-- When the paint doesn't have any layers just render the content with filters -->
		<span class="layer" style:filter>
			{@render children()}
		</span>
	{:else}
		<!-- Only apply filters to first layer -->
		{#each layers as { opacity, image, color }, i}
			<span
				class="layer bg-clip"
				style:opacity
				style:background-image={image}
				style:background-color={color}
				style:filter={i === 0 ? filter : undefined}
			>
				{@render children()}
			</span>
		{/each}
	{/if}
{/snippet}

{#if enableDialog}
	<!-- Making this a real button sadly breaks the paint rendering -->
	<div
		role="button"
		tabindex="-1"
		class="paint"
		title="Paint: {paint.name.length > 0 ? paint.name : paint.id}"
		onclick={showDialog}
		{...restProps}
	>
		{@render content()}
	</div>
{:else}
	<div class="paint" title="Paint: {paint.name.length > 0 ? paint.name : paint.id}" {...restProps}>
		{@render content()}
	</div>
{/if}

<style lang="scss">
	.paint {
		display: grid;
		justify-items: start;
	}

	.layer {
		// Overlay all layers on top of each other
		grid-area: 1 / 1 / -1 / -1;

		&.bg-clip {
			background-color: currentColor;

			-webkit-text-fill-color: transparent;
			background-clip: text;
			-webkit-background-clip: text;
			background-size: cover;
		}
	}

	div[role="button"] {
		cursor: pointer;
	}
</style>
