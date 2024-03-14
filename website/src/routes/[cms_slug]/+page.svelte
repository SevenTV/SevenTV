<script lang="ts">
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

<svelte:head>
	<title>{data.slug} - 7TV</title>
</svelte:head>

<h1>{data.slug}</h1>
