<script lang="ts">
	import { page } from "$app/stores";
	import { createEventDispatcher } from "svelte";
	import Button from "./button.svelte";

	const dispatch = createEventDispatcher();

	export let title: string | null = null;
	export let href: string | null = null;
	export let big: boolean = false;
	export let responsive: boolean = false;
	export let matcher: (id: string | null, url: URL, href: string | null) => boolean = (_id, url, href) => {
		return url.pathname === href;
	};

	function scrollIntoView(e: MouseEvent) {
		if (e.target instanceof HTMLElement) {
			e.target.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
		}
		dispatch("click");
	}

	$: active = matcher($page.route.id, $page.url, href);
</script>

{#if responsive}
	<Button
		{href}
		{big}
		secondary={active}
		draggable="false"
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
		secondary={active}
		draggable="false"
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
		secondary={active}
		draggable="false"
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
