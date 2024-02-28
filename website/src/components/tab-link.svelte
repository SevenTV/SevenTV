<script lang="ts">
	import HideOn from "./hide-on.svelte";
	import { page } from "$app/stores";
	import Button from "./button.svelte";

	export let title: string | null = null;
	export let href: string;
	export let big: boolean = false;
	export let responsive: boolean = false;
	export let matcher: (id: string | null, url: URL, href: string) => boolean = (_id, url, href) => {
		return url.pathname === href;
	};

	function scrollIntoView(e: MouseEvent) {
		if (e.target instanceof HTMLElement) {
			e.target.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
		}
	}

	$: active = matcher($page.route.id, $page.url, href);
</script>

{#if responsive}
	<Button
		{href}
		{big}
		draggable="false"
		primary={active}
		style={!active && "color: var(--text-light)"}
		on:click={scrollIntoView}
		hideOnMobile
	>
		<svelte:fragment slot="icon">
			{#if active}
				<slot name="active" />
			{:else}
				<slot />
			{/if}
		</svelte:fragment>
		{title}
	</Button>
	<Button
		{href}
		{big}
		draggable="false"
		primary={active}
		style={!active && "color: var(--text-light)"}
		on:click={scrollIntoView}
		hideOnDesktop
	>
		<svelte:fragment slot="icon">
			{#if active}
				<slot name="active" />
			{:else}
				<slot />
			{/if}
		</svelte:fragment>
	</Button>
{:else}
	<Button
		{href}
		{big}
		draggable="false"
		primary={active}
		style={!active && "color: var(--text-light)"}
		on:click={scrollIntoView}
	>
		<svelte:fragment slot="icon">
			{#if active}
				<slot name="active" />
			{:else}
				<slot />
			{/if}
		</svelte:fragment>
		{title}
	</Button>
{/if}