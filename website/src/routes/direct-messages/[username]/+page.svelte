<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import { CaretLeft } from "phosphor-svelte";
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

<svelte:head>
	<title>Direct Messages - 7TV</title>
</svelte:head>

<Button href="/direct-messages">
	<CaretLeft slot="icon" />
	Back
</Button>
<h1>DMs with {data.username}</h1>
