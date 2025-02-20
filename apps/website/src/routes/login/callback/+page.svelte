<script lang="ts">
	import type { PageData } from "./$types";
	import Spinner from "$/components/spinner.svelte";
	import { sessionToken } from "$/lib/auth";
	import { Check, X } from "phosphor-svelte";
	import { goto } from "$app/navigation";
	import { purchasePickems } from "$/lib/pickems";

	let { data }: { data: PageData } = $props();

	let login = $derived(
		data.streamed.loginRequest.then((token) => {
			sessionToken.set(token);

			let payload = null;
			const splitToken = token.split(".");
			if (splitToken[1]) {
				try {
					payload = JSON.parse(atob(splitToken[1]));
				} catch (e) {
					console.error(e);
				}
			}
			if (data.returnTo?.includes("?")) {
				const params = new URLSearchParams(data.returnTo.slice(data.returnTo.indexOf("?")));
				if (params.has("pickems")) {
					const productId = params.get("pickems")!;
					purchasePickems(productId === "undefined" ? undefined : productId, true);
				}
			}

			if (data.returnTo) {
				goto(data.returnTo);
			} else if (payload?.sub) {
				goto(`/users/${payload.sub}`);
			} else {
				goto("/");
			}
		}),
	);
</script>

<svelte:head>
	{#await login}
		<title>Logging you in...</title>
	{:then _}
		<title>Logged in</title>
	{:catch _}
		<title>Login Failed</title>
	{/await}
</svelte:head>

<div class="container">
	{#await login}
		<Spinner />
		<h2>Logging you in...</h2>
	{:then _}
		<Check />
		<h2>Logged in</h2>
	{:catch e}
		<X />
		<h2>Login Failed</h2>
		<p>{e}</p>
	{/await}
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
