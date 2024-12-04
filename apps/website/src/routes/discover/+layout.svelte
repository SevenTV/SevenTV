<script lang="ts">
	import TextInput from "$/components/input/text-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { Heart, MagnifyingGlass, Newspaper } from "phosphor-svelte";
	import type { Snippet } from "svelte";
	import { t } from "svelte-i18n";
	import { type Page } from "@sveltejs/kit";

	let { children }: { children: Snippet } = $props();

	function matcher(page: Page, href: string | undefined) {
		if (href === "/discover") return page.url.pathname === href;
		return !!href && page.url.pathname.startsWith(href);
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.discover.title")}</h1>
		<nav class="link-list">
			<TabLink href="/discover" title={$t("common.news")} big {matcher}>
				<Newspaper />
				{#snippet active()}
					<Newspaper weight="fill" />
				{/snippet}
			</TabLink>
			<TabLink href="/discover/following" title={$t("common.following")} big {matcher}>
				<Heart />
				{#snippet active()}
					<Heart weight="fill" />
				{/snippet}
			</TabLink>
		</nav>
		<hr class="hide-on-mobile" />
		<div class="followed hide-on-mobile">
			<h2>{$t("pages.discover.followed")}</h2>
			<TextInput placeholder={$t("labels.search")}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput>
			<div class="channels">
				<!-- {#each Array(5) as _, i}
					<ChannelPreview size={1.5} index={i} user="followed{i}" />
				{/each} -->
			</div>
		</div>
	</aside>
	<div class="content">
		{@render children()}
	</div>
</div>

<style lang="scss">
	.content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.followed {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;

		h2 {
			font-size: 0.875rem;
			font-weight: 600;
		}

		.channels {
			display: flex;
			flex-direction: column;
			gap: 0.25rem;

			margin-inline: -0.5rem;
		}
	}
</style>
