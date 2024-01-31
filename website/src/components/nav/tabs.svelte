<script lang="ts">
	import { page } from "$app/stores";

	type Tab = {
		name: string;
		pathname: string;
		hightlight?: boolean;
	};

	export let tabs: Tab[];
</script>

<ul aria-label="tabs">
	{#each tabs as tab}
		<li aria-current={$page.url.pathname.startsWith(tab.pathname)}>
			<a
				class:selected={$page.url.pathname.startsWith(tab.pathname)}
				class:highlight={tab.hightlight}
				href={tab.pathname}
				draggable="false">{tab.name}</a
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

			&.highlight {
				color: var(--subscriber);
			}

			&:hover,
			&:focus-visible {
				border-bottom-color: var(--secondary-hover);
			}

			&:active {
				border-bottom-color: var(--primary-active);
			}

			&.selected {
				border-bottom-color: var(--primary);
			}
			&.selected.highlight {
				border-bottom-color: var(--subscriber);
			}
		}
	}
</style>
