<script lang="ts">
	import Spinner from "$/components/spinner.svelte";
	import { sessionToken } from "$/lib/auth";
	import { signInDialogMode } from "$/lib/layout";
	import { browser } from "$app/environment";
	import { Check, X } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	let state = $state<"loading" | "failed" | "success">("loading");

	$effect(() => {
		if ($sessionToken) {
			if (browser && window.opener) {
				window.opener.postMessage(
					{ type: "7tv-token", token: $sessionToken },
					"https://www.twitch.tv",
				);
				state = "success";
				window.close();
			} else {
				state = "failed";
			}
		} else if ($sessionToken === null) {
			$signInDialogMode = "shown-without-close";
		}
	});
</script>

<svelte:head>
	{#if state === "loading"}
		<title>{$t("pages.auth.title.logging")}</title>
	{:else if state === "success"}
		<title>{$t("pages.auth.title.logged")}</title>
	{:else if state === "failed"}
		<title>{$t("pages.auth.title.failed")}</title>
	{/if}
</svelte:head>

<div class="container">
	{#if state === "loading"}
		<Spinner />
		<h2>{$t("pages.auth.state.logging")}</h2>
	{:else if state === "success"}
		<Check />
		<h2>{$t("pages.auth.state.logged")}</h2>
	{:else if state === "failed"}
		<X />
		<h2>{$t("pages.auth.state.failed.title")}</h2>
		<p>{$t("pages.auth.state.failed.message")}</p>
	{/if}
</div>

<style lang="scss">
	.container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;

		height: 100%;
	}
</style>
