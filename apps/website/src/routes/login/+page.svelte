<script lang="ts">
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { user } from "$/lib/auth";
	import { Check } from "phosphor-svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("pages.login.title")}</title>
</svelte:head>

<div class="container">
	{#if $user === undefined}
		<Spinner />
	{:else if $user === null}
		<SignInDialog mode={"shown-without-close"} />
	{:else}
		<Check />
		<h2>{$t("pages.login.logged_in")}</h2>
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
