<script lang="ts">
	import { page } from "$app/stores";
	import Button from "./button.svelte";

	export let title: string;
	export let href: string;

	function scrollIntoView(e: MouseEvent) {
		if (e.target instanceof HTMLElement) {
			e.target.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
		}
	}
</script>

<Button
	{href}
	draggable="false"
	primary={$page.url.pathname === href}
	style={$page.url.pathname !== href && "color: var(--text-lighter)"}
	on:click={scrollIntoView}
>
	<svelte:fragment slot="icon">
		{#if $page.url.pathname === href}
			<slot name="active" />
		{:else}
			<slot />
		{/if}
	</svelte:fragment>
	{title}
</Button>
