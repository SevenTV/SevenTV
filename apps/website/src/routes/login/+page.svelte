<script lang="ts">
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { user } from "$/lib/auth";
	import { Check } from "phosphor-svelte";
</script>

<svelte:head>
	<title>Login</title>
</svelte:head>

<div class="container">
	{#if $user === undefined}
		<Spinner />
	{:else if $user === null}
		<SignInDialog mode={"shown-without-close"} />
	{:else}
		<Check />
		<h2>Logged in</h2>
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
