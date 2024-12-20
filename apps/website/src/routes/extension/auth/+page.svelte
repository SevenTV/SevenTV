<script lang="ts">
	import Spinner from "$/components/spinner.svelte";
	import { sessionToken } from "$/lib/auth";
	import { signInDialogMode } from "$/lib/layout";
	import { browser } from "$app/environment";
	import { Check, X } from "phosphor-svelte";

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
		<title>Logging you in...</title>
	{:else if state === "success"}
		<title>Logged in</title>
	{:else if state === "failed"}
		<title>Failed to log in</title>
	{/if}
</svelte:head>

<div class="container">
	{#if state === "loading"}
		<Spinner />
		<h2>Logging you in...</h2>
	{:else if state === "success"}
		<Check />
		<h2>Logged in</h2>
	{:else if state === "failed"}
		<X />
		<h2>Login Failed</h2>
		<p>Make sure to open this window through the extension</p>
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
