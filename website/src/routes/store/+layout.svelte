<script lang="ts">
	import { PaintBrush, Star, TShirt } from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>Store</h1>
		<nav class="link-list">
			<TabLink href="/store" title="Subscription" big>
				<Star />
				<Star weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/store/paint-bundles" title="Paint Bundles" big>
				<PaintBrush />
				<PaintBrush weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/store/merch" title="Merch" big>
				<TShirt />
				<TShirt weight="fill" slot="active" />
			</TabLink>
		</nav>
		<hr />
		<label class="redeem">
			Redeem Gift Code
			<input type="text" placeholder="Enter code" />
		</label>
	</aside>
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.redeem {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		color: var(--text-light);
		font-size: 0.75rem;
		font-weight: 500;
	}

	// Only desktop
	@media screen and (min-width: 961px) {
		.content {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;

			& > :global(*) {
				// right margin because of side-bar-layout
				margin-right: 1.25rem;
			}
		}
	}
</style>
