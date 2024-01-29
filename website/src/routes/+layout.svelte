<script lang="ts">
	import "$/styles/fonts.scss";
	import "$/styles/colors.scss";
	import "$/styles/global.scss";
	import TopNav from "$/components/nav/top-nav.svelte";
	import { showMobileMenu } from "$/lib/stores";
	import MobileMenu from "$/components/nav/mobile-menu.svelte";
	import { beforeNavigate } from "$app/navigation";

	beforeNavigate((nav) => {
		// Hide menu on navigate
		nav.complete.then(() => {
			$showMobileMenu = false;
		});
	});
</script>

<header>
	<a href="#main" class="skip-to-main">Skip to main content</a>
	<TopNav />
</header>

<main id="main">
	{#if $showMobileMenu}
		<MobileMenu />
	{:else}
		<slot />
	{/if}
</main>

<style lang="scss">
	:global(body) {
		max-height: 100vh; /* For browsers that don't support svh */
		max-height: 100svh;
		min-height: 100vh; /* For browsers that don't support svh */
		min-height: 100svh;

		display: grid;
		grid-template-rows: auto 1fr;
	}

	header {
		display: contents;
	}

	main {
		overflow: auto;
	}

	.skip-to-main {
		position: absolute;
		color: var(--primary);
		opacity: 0;
		pointer-events: none;

		&:focus-visible {
			opacity: 1;
			pointer-events: unset;
		}
	}
</style>
