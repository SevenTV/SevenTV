<script lang="ts">
	import { Layout } from "$/store/layout";

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
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		grid-template-rows: repeat(auto-fill, 10rem);
		place-items: center;
		gap: 1rem;

		&.scrollable {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;
		}

		&.small-grid {
			grid-template-columns: repeat(auto-fill, minmax(5rem, 1fr));
			grid-template-rows: repeat(auto-fill, 5rem);
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
