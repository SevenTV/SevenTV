<script lang="ts">
	import { page } from "$app/stores";

	export let title: string;
	export let href: string;

	export let first: boolean = false;
	export let last: boolean = false;

	function scrollIntoView(e: MouseEvent) {
		if (e.target instanceof HTMLElement) {
			e.target.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
		}
	}
</script>

<a
	{href}
	draggable="false"
	class:selected={$page.url.pathname === href}
	class:first
	class:last
	on:click={scrollIntoView}
>
	{#if $page.url.pathname === href}
		<slot name="active" />
	{:else}
		<slot />
	{/if}
	{title}
</a>

<style lang="scss">
	a {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		padding-left: 0.75rem;
		border-radius: 0.5rem;

		text-decoration: none;
		color: var(--text-lighter);
		font-size: 0.875rem;
		font-weight: 600;

		user-select: none;

		&.selected {
			color: var(--text);
			background-color: var(--primary);
		}

		&:hover, &:focus-visible {
			background-color: var(--primary-hover);
			text-decoration: none;
		}

		&:active {
			background-color: var(--primary-active);
		}
	}
</style>
