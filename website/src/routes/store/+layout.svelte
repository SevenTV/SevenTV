<script lang="ts">
	import { PaintBrush, Star, TShirt } from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>Store</h1>
		<div class="link-list">
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
		</div>
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
	.side-bar-layout {
		position: relative;
	}

	.side-bar {
		z-index: 1;

		h1 {
			font-size: 1.125rem;
			font-weight: 700;
			margin: 0.25rem 0;
		}

		.link-list {
			display: flex;
			flex-direction: column;
			gap: 0.25rem;
		}

		.redeem {
			display: flex;
			flex-direction: column;
			gap: 0.5rem;

			color: var(--text-light);
			font-size: 0.75rem;
			font-weight: 500;
		}
	}

	.content {
		overflow: auto;
	}
</style>
