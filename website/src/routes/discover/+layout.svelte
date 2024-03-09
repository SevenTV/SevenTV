<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { Heart, Newspaper } from "phosphor-svelte";

	function matcher(_id: string | null, url: URL, href: string | null) {
		if (href === "/discover") return url.pathname === href;
		return !!href && url.pathname.startsWith(href);
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>Discover</h1>
		<nav class="link-list">
			<TabLink href="/discover" title="News" big {matcher}>
				<Newspaper />
				<Newspaper weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/discover/following" title="Following" big {matcher}>
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
