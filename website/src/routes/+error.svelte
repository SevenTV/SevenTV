<script lang="ts">
	import Troll from "$/components/icons/troll.svelte";
	import { page } from "$app/stores";

	let title = $page.error?.message ?? "Error";
</script>

<svelte:head>
	<title>{title} - 7TV</title>
</svelte:head>

<div class="container">
	<div class="troll">
		<Troll />
	</div>
	<h1>{title}</h1>
	<span>
		{#if $page.status === 401}
			You are unauthorized to view this page.
		{:else if $page.status === 403}
			You do not have permission to view this page.
		{:else if $page.status === 404}
			The page you were looking for could not be found.
		{:else}
			An internal error occurred.
			<br />
			You can try refreshing the page.
			<br />
			If the problem persists, please contact us.
		{/if}
	</span>
	<a href="/">Go Back</a>
</div>

<style lang="scss">
	.container {
		height: 100%;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		text-align: center;

		font-size: 1.25rem;
	}

	.troll {
		font-size: 12rem;
	}

	a {
		color: var(--text);
	}
</style>
