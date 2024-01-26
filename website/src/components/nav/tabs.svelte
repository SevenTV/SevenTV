<script lang="ts">
	import { page } from "$app/stores";

	type Tab = {
		name: string;
		pathname: string;
	};

	export let tabs: Tab[];
</script>

<ul class="selector" aria-label="Tab selector">
	{#each tabs as tab}
        <li>
            <a class:selected={$page.url.pathname === tab.pathname} href={tab.pathname} draggable="false">{tab.name}</a>
        </li>
	{/each}
</ul>

<style lang="scss">
	.selector {
		display: flex;
		flex-wrap: wrap;
        align-items: center;

		user-select: none;

        li {
            display: contents;
        }

		a {
            padding: 0.75rem 1rem;
			border-top: 2px solid transparent;
			border-bottom: 2px solid transparent;
			background-color: none;
			font-weight: 600;
			color: var(--text);
			text-decoration: none;

            &:hover, &:focus-visible {
                border-bottom: 2px solid var(--text-light);
            }

            &.selected {
                border-bottom: 2px solid var(--primary);
            }
		}
	}
</style>
