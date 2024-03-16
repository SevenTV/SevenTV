<script lang="ts">
	import ChannelPreview from "$/components/channel-preview.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { Heart, MagnifyingGlass, Newspaper } from "phosphor-svelte";
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
		<hr class="hide-on-mobile" />
		<div class="followed hide-on-mobile">
			<h2>{$t("pages.discover.followed")}</h2>
			<TextInput placeholder={$t("labels.search")}>
				<MagnifyingGlass slot="icon" />
			</TextInput>
			<div class="channels">
				{#each Array(5) as _, i}
					<ChannelPreview size={1.5} index={i} user="followed{i}" />
				{/each}
			</div>
		</div>
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
