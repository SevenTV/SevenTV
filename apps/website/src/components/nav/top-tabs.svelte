<script lang="ts">
	import { page } from "$app/stores";
	import { ArrowSquareOut } from "phosphor-svelte";

	type Tab = {
		name: string;
		pathname: string;
		highlight?: string;
		arrow?: boolean;
	};

	let { tabs }: { tabs: Tab[] } = $props();
</script>

<ul aria-label="tabs">
	{#each tabs as tab}
		<li aria-current={$page.url.pathname.startsWith(tab.pathname)}>
			<a
				class:selected={$page.url.pathname.startsWith(tab.pathname)}
				style="--highlight: {tab.highlight}"
				href={tab.pathname}
				draggable="false"
				>{tab.name}
				{#if tab.arrow}
					<ArrowSquareOut style="vertical-align: bottom;" />
				{/if}</a
			>
		</li>
	{/each}
</ul>

<style lang="scss">
	ul {
		display: flex;
		align-items: center;

		user-select: none;

		li {
			display: contents;
		}

		a {
			padding: 1rem;
			border-top: 2px solid transparent;
			border-bottom: 2px solid transparent;
			background-color: none;
			font-weight: 600;
			color: var(--text);
			text-decoration: none;

			text-overflow: ellipsis;
			white-space: nowrap;
			overflow: hidden;

			transition: border-bottom-color 0.1s;

			color: var(--highlight, --text);

			&:hover,
			&:focus-visible {
				border-bottom-color: var(--secondary-hover);
			}

			&:active {
				border-bottom-color: var(--highlight, --primary-active);
			}

			&.selected {
				border-bottom-color: var(--highlight, --primary);
			}
		}
	}
</style>
