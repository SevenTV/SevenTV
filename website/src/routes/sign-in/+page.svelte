<script lang="ts">
	import { user } from "$/lib/stores";
</script>

<svelte:head>
	<title>Sign In - 7TV</title>
</svelte:head>

<h1>Sign In</h1>

<button class="button primary" on:click={() => ($user = true)}>Fake login xdd</button>
