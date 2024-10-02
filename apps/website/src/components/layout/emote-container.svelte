<script lang="ts">
	import { Layout } from "$/store/layout";

	export let scrollable = false;
	export let layout: Layout | null = null;
	export let clientWidth: number | undefined;
	export let clientHeight: number | undefined;
</script>

<div
	class="emotes"
	class:small-grid={layout === Layout.SmallGrid}
	class:scrollable
	bind:clientWidth={clientWidth}
	bind:clientHeight={clientHeight}
	{...$$restProps}
>
	<slot />
</div>

<style lang="scss">
	.emotes {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		place-items: center;
		gap: 1rem;

		&.scrollable {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;
		}

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
