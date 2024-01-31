<script lang="ts">
	import { page } from "$app/stores";
	import type { IconDefinition as SolidIcon } from "@fortawesome/pro-solid-svg-icons";
	import type { IconDefinition as RegularIcon } from "@fortawesome/pro-regular-svg-icons";
	import Fa from "svelte-fa";

	export let title: string;
	export let icon: SolidIcon | RegularIcon;
	export let activeIcon: SolidIcon | RegularIcon;
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
	<Fa icon={$page.url.pathname === href ? activeIcon : icon} size="1.2x" />
	{title}
</a>
