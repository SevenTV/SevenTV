<script lang="ts">
	import { page } from "$app/stores";

	type Tab = {
		name: string;
		pathname: string;
	};

	export let tabs: Tab[];
</script>

<ul aria-label="tabs">
	{#each tabs as tab}
        <li aria-current={$page.url.pathname === tab.pathname}>
            <a class:selected={$page.url.pathname === tab.pathname} href={tab.pathname} draggable="false">{tab.name}</a>
        </li>
	{/each}
</ul>

<style lang="scss">
	ul {
		display: flex;
		flex-wrap: wrap;
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

			transition: border-bottom-color 0.1s;

            &:hover, &:focus-visible {
                border-bottom-color: var(--text-light);
            }

            &.selected {
                border-bottom-color: var(--primary);
            }
		}
	}
</style>
