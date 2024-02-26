<script lang="ts">
	import { page } from "$app/stores";
	import Button from "./button.svelte";

	type Tab = {
		name: string;
		pathname: string;
	};

	export let tabs: Tab[];
</script>

<ul aria-label="tabs">
	{#each tabs as tab}
		<li aria-current={$page.url.pathname === tab.pathname}>
			<Button primary href={tab.pathname} draggable="false" style={$page.url.pathname === tab.pathname ? "color: var(--text); background-color: var(--primary-light);" : "color: var(--text-lighter);"}>
				{tab.name}
			</Button>
		</li>
	{/each}
</ul>

<style lang="scss">
	ul {
		display: flex;
		align-items: center;

		user-select: none;

		background-color: var(--primary);
		border-radius: 0.5rem;

		li {
			display: contents;
		}
	}
</style>
