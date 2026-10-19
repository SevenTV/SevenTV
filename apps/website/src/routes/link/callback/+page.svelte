<script lang="ts">
	import type { PageData } from "./$types";
	import Spinner from "$/components/spinner.svelte";
	import { Check, X } from "phosphor-svelte";
	import { goto } from "$app/navigation";

	let { data }: { data: PageData } = $props();

	let login = $derived(
		data.streamed.linkRequest.then(() => {
			if (data.returnTo) {
				goto(data.returnTo);
			} else {
				goto("/settings");
			}
		}),
	);
</script>

<svelte:head>
	{#await login}
		<title>Linking Your Account...</title>
	{:then _}
		<title>Account Successfully Linked</title>
	{:catch _}
		<title>Linking Failed</title>
	{/await}
</svelte:head>

<div class="container">
	{#await login}
		<Spinner />
		<h2>Linking Your Account...</h2>
	{:then _}
		<Check />
		<h2>Account Successfully Linked</h2>
	{:catch e}
		<X />
		<h2>Linking Failed</h2>
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
