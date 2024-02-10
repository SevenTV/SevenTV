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
			<a
				class="button secondary"
				class:selected={$page.url.pathname === tab.pathname}
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

		background-color: var(--secondary);
		border-radius: 0.5rem;

		li {
			display: contents;
		}

		a {
			color: var(--text-lighter);

			&.selected {
				color: var(--text);
				background-color: var(--secondary-light);
			}
		}
	}
</style>
