<script lang="ts">
	import { Gift, PaintBrush, Star } from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.store.title")}</h1>
		<nav class="link-list">
			<TabLink href="/store" title={$t("common.subscriptions", { values: { count: 1 } })} big>
				<Star />
				{#snippet active()}
					<Star weight="fill" />
				{/snippet}
			</TabLink>
			<TabLink href="/store/paint-bundles" title={$t("common.paint_bundles")} big>
				<PaintBrush />
				{#snippet active()}
					<PaintBrush weight="fill" />
				{/snippet}
			</TabLink>
			<TabLink href="/store/redeem" title="Redeem" big>
				<Gift />
				{#snippet active()}
					<Gift weight="fill" />
				{/snippet}
			</TabLink>
		</nav>
	</aside>
	<div class="content">
		{@render children()}
	</div>
</div>

<style lang="scss">
	// Only desktop
	@media screen and (min-width: 961px) {
		.content {
			overflow: auto;
			scrollbar-gutter: stable;

			& > :global(*) {
				// right margin because of side-bar-layout
				margin-right: 1.25rem;
			}
		}
	}
</style>
