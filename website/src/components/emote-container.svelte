<script lang="ts">
	import { Layout } from "$/lib/stores";

	export let scrollable = false;
	export let layout: Layout | null = null;
</script>

<div
	class="emotes"
	class:small-grid={layout === Layout.SmallGrid}
	class:scrollable
	{...$$restProps}
>
	<slot />
</div>

<style lang="scss">
	.emotes {
		&.scrollable {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;
		}

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		place-items: center;
		gap: 1rem;

		&.small-grid {
			grid-template-columns: repeat(auto-fill, minmax(5rem, 1fr));
			gap: 0.5rem;
		}
	}

	@media screen and (max-width: 960px) {
		.emotes {
			grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));
			gap: 0.5rem;
		}
	}
</style>
