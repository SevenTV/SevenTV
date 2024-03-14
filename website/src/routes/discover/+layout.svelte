<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { Heart, Newspaper } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	function matcher(_id: string | null, url: URL, href: string | null) {
		if (href === "/discover") return url.pathname === href;
		return !!href && url.pathname.startsWith(href);
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.discover.title")}</h1>
		<nav class="link-list">
			<TabLink href="/discover" title={$t("common.news")} big {matcher}>
				<Newspaper />
				<Newspaper weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/discover/following" title={$t("common.following")} big {matcher}>
				<Heart />
				<Heart weight="fill" slot="active" />
			</TabLink>
		</nav>
	</aside>
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
