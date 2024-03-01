<script lang="ts">
	import { Layout } from "$/lib/stores";

	export let scrollable = false;
	export let topMargin: number = 0;
	export let layout: Layout | null = null;

	$: size = layout ? (layout === Layout.BigGrid ? 10 : 5) : 10;

	function style(topMargin: number, size: number | null) {
		let style = `margin-top: ${topMargin}rem; `;
		if (size) {
			style += `--size: ${size}rem; `;
		}
		return style;
	}
</script>

<div class="emotes" style={style(topMargin, size)} class:scrollable>
	<slot />
</div>

<style lang="scss">
	.emotes {
		&.scrollable {
			overflow: overlay;
			scrollbar-gutter: stable;
		}

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(var(--size, 10rem), 1fr));
		place-items: center;
		gap: 1rem;
	}

	@media screen and (max-width: 960px) {
		.emotes {
			grid-template-columns: repeat(auto-fill, minmax(var(--size, 8rem), 1fr));
			gap: 0.5rem;
		}
	}
</style>
