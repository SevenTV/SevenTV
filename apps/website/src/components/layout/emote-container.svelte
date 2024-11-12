<script lang="ts">
	import { type Layout } from "$/lib/layout";
	import type { Snippet } from "svelte";
	import { type HTMLAttributes } from "svelte/elements";

	type Props = {
		scrollable?: boolean;
		layout?: Layout;
		children?: Snippet;
	} & HTMLAttributes<HTMLDivElement>;

	let { scrollable, layout, children, ...restProps }: Props = $props();
</script>

<div
	class="emotes"
	class:small-grid={layout === "small-grid"}
	class:scrollable
	class:scrollable-on-desktop={scrollable === undefined}
	{...restProps}
>
	{@render children?.()}
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

	@media screen and (min-width: 961px) {
		.emotes.scrollable-on-desktop {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;
		}
	}
</style>
