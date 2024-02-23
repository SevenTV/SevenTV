<script lang="ts">
	import { page } from "$app/stores";

	export let title: string;
	export let href: string;

	function scrollIntoView(e: MouseEvent) {
		if (e.target instanceof HTMLElement) {
			e.target.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
		}
	}
</script>

<a
	{href}
	class="button icon-left no-bg"
	draggable="false"
	class:secondary={$page.url.pathname === href}
	on:click={scrollIntoView}
>
	{#if $page.url.pathname === href}
		<slot name="active" />
	{:else}
		<slot />
	{/if}
	{title}
</a>
