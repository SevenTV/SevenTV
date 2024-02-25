<script lang="ts">
	import Button from "$/components/button.svelte";
import { user } from "$/lib/stores";
</script>

<svelte:head>
	<title>Sign In - 7TV</title>
</svelte:head>

<h1>Sign In</h1>

<Button secondary on:click={() => ($user = true)}>
	Fake login xdd
</Button>
