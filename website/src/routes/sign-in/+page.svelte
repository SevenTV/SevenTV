<script lang="ts">
	import Button from "$/components/button.svelte";
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
</script>

<svelte:head>
	<title>Sign In - 7TV</title>
</svelte:head>

<h1>Sign In</h1>

<Button primary on:click={login}>Fake login xdd</Button>
