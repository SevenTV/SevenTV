<svelte:options runes={true} />

<script lang="ts">
	import "$/styles/fonts.scss";
	import "$/styles/variables.scss";
	import "$/styles/global.scss";
	import "$/lib/i18n";
	import TopNav from "$/components/nav/top-nav.svelte";
	import {
		showMobileMenu,
		signInDialogMode,
		uploadDialogMode,
		defaultEmoteSetDialogMode,
	} from "$/lib/layout";
	import Menu from "$/components/nav/menu.svelte";
	import { beforeNavigate } from "$app/navigation";
	import { IconContext, Warning } from "phosphor-svelte";
	import UploadDialog from "$/components/dialogs/upload-dialog.svelte";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import DefaultEmoteSetDialog from "$/components/dialogs/default-emote-set-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import ErrorDialog from "$/components/dialogs/error-dialog.svelte";
	import { currentError, errorDialogMode } from "$/lib/error";
	import "$/lib/emoteSets";

	let { children }: { children: Snippet } = $props();

	beforeNavigate((nav) => {
		// Hide menu on navigate
		nav.complete.then(() => {
			$showMobileMenu = false;
		});
	});
</script>

<IconContext values={{ size: 1.2 * 16, weight: "bold", style: "flex-shrink: 0" }}>
	<header>
		<a href="#main" class="skip-to-main">{$t("common.skip_to_content")}</a>
		<div class="alert-bar">
			<Warning />
			<span>Under construction – Read-only</span>
			<span class="small">Some features might be unavailable.</span>
		</div>
		<TopNav />
	</header>

	<UploadDialog bind:mode={$uploadDialogMode} />
	<SignInDialog bind:mode={$signInDialogMode} />
	<DefaultEmoteSetDialog bind:mode={$defaultEmoteSetDialogMode} />
	<ErrorDialog bind:mode={$errorDialogMode} error={$currentError} />
	<main id="main">
		{#if $showMobileMenu}
			<Menu onCloseRequest={() => ($showMobileMenu = false)} />
		{:else}
			{@render children()}
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
		grid-template-rows: auto auto 1fr;
	}

	header {
		display: contents;
	}

	.alert-bar {
		background-color: var(--bg-light);
		padding: 0.4rem;

		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		flex-wrap: wrap;

		color: var(--text);
		font-weight: 600;

		.small {
			font-weight: 400;
			font-size: 0.9rem;
		}
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
