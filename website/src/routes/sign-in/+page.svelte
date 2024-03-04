<script lang="ts">
	import Button from "$/components/button.svelte";
	import CartDialog from "$/components/dialogs/cart-dialog.svelte";
	import EditEmoteDialog from "$/components/dialogs/edit-emote-dialog.svelte";
	import EditEmoteSetDialog from "$/components/dialogs/edit-emote-set-dialog.svelte";
	import { user } from "$/lib/stores";
	import { goto } from "$app/navigation";
	import type { PageData } from "./$types";

	export let data: PageData;

	function login() {
		$user = true;
		if (data.redirect) {
			goto(data.redirect);
		}
	}

	let cartOpen = false;
	let editEmoteOpen = false;
	let editEmoteSetOpen = false;
</script>

<svelte:head>
	<title>Sign In - 7TV</title>
</svelte:head>

<h1>Sign In</h1>

<Button primary on:click={login}>Fake login xdd</Button>

<Button primary on:click={() => (cartOpen = true)}>Open cart modal</Button>

{#if cartOpen}
	<CartDialog on:close={() => (cartOpen = false)} />
{/if}

<Button primary on:click={() => (editEmoteOpen = true)}>Open edit emote modal</Button>

{#if editEmoteOpen}
	<EditEmoteDialog on:close={() => (editEmoteOpen = false)} />
{/if}

<Button primary on:click={() => (editEmoteSetOpen = true)}>Open edit emote set modal</Button>

{#if editEmoteSetOpen}
	<EditEmoteSetDialog on:close={() => (editEmoteSetOpen = false)} />
{/if}
