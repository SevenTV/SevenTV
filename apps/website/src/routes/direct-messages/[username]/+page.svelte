<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import { CaretLeft } from "phosphor-svelte";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();
</script>

<Button href="/direct-messages">
	{#snippet icon()}
		<CaretLeft />
	{/snippet}
	Back
</Button>
<h1>DMs with {data.username}</h1>
