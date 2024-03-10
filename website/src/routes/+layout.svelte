<script lang="ts">
	import "$/styles/fonts.scss";
	import "$/styles/variables.scss";
	import "$/styles/global.scss";
	import TopNav from "$/components/nav/top-nav.svelte";
	import {
		showMobileMenu,
		signInDialogMode,
		uploadDialogMode,
		defaultEmoteSetDialogMode,
	} from "$/lib/stores";
	import Menu from "$/components/nav/menu.svelte";
	import { beforeNavigate } from "$app/navigation";
	import { IconContext } from "phosphor-svelte";
	import UploadDialog from "$/components/dialogs/upload-dialog.svelte";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import { onMount } from "svelte";
	import moment from "moment";
	import DefaultEmoteSetDialog from "$/components/dialogs/default-emote-set-dialog.svelte";

	onMount(() => {
		// Set moment locale
		moment.locale("en-US");
	});

	beforeNavigate((nav) => {
		// Hide menu on navigate
		nav.complete.then(() => {
			$showMobileMenu = false;
		});
	});
</script>

<IconContext values={{ size: "1.2rem", weight: "bold", style: "flex-shrink: 0" }}>
	<header>
		<a href="#main" class="skip-to-main">Skip to main content</a>
		<TopNav />
	</header>

	<UploadDialog bind:mode={$uploadDialogMode} />
	<SignInDialog bind:mode={$signInDialogMode} />
	<DefaultEmoteSetDialog bind:mode={$defaultEmoteSetDialogMode} />
	<dialog>Test</dialog>
	<main id="main">
		{#if $showMobileMenu}
			<Menu />
		{:else}
			<slot />
		{/if}
	</main>
</IconContext>

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
		overflow: overlay;
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
