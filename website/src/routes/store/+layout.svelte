<script lang="ts">
	import { PaintBrush, Star } from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { t } from "svelte-i18n";
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.store.title")}</h1>
		<nav class="link-list">
			<TabLink href="/store" title={$t("common.subscriptions", { values: { count: 1 } })} big>
				<Star />
				<Star weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/store/paint-bundles" title={$t("common.paint_bundles")} big>
				<PaintBrush />
				<PaintBrush weight="fill" slot="active" />
			</TabLink>
		</nav>
		<hr />
		<label class="redeem">
			{$t("pages.store.redeem")}
			<input type="text" placeholder={$t("labels.redeem")} />
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
