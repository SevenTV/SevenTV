<script lang="ts">
	import Developers from "$/components/landing/developers.svelte";
	import Download from "$/components/landing/download.svelte";
	import Features from "$/components/landing/features.svelte";
	import Footer from "$/components/landing/footer.svelte";
	import Hero from "$/components/landing/hero.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.suffix")}</title>
</svelte:head>

<div class="layout">
	<Hero />
	<Download />
	<Features />
	<Developers />
	<Footer />
</div>

<style lang="scss">
	// Akony Font
	@font-face {
		font-display: swap;
		font-family: "AKONY";
		font-style: normal;
		font-weight: 400;
		src:
			url("$assets/fonts/akony.woff2") format("woff2"),
			url("$assets/fonts/akony.ttf") format("truetype");
	}

	.layout {
		padding: 1rem 2rem 0rem 2rem;

		display: flex;
		flex-direction: column;
		gap: 10rem;
	}
</style>
