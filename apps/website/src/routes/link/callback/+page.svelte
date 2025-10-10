<script lang="ts">
	import type { PageData } from "./$types";
	import Spinner from "$/components/spinner.svelte";
	import { Check, X } from "phosphor-svelte";
	import { goto } from "$app/navigation";
	import { t } from "svelte-i18n";

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
		<title>{$t("pages.link.linking")}</title>
	{:then _}
		<title>{$t("pages.link.linking_successful")}</title>
	{:catch _}
		<title>{$t("pages.link.linking_failed")}</title>
	{/await}
</svelte:head>

<div class="container">
	{#await login}
		<Spinner />
		<h2>{$t("pages.link.linking")}</h2>
	{:then _}
		<Check />
		<h2>{$t("pages.link.linking_successful")}</h2>
	{:catch e}
		<X />
		<h2>{$t("pages.link.linking_failed")}</h2>
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
