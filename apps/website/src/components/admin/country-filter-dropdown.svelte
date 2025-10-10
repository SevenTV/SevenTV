<script lang="ts">
	import { fade } from "svelte/transition";
	import { CaretDown, CaretRight, Check, GlobeHemisphereWest } from "phosphor-svelte";
	import CountryFlag from "../country-flag.svelte";
	import { countriesFilter } from "$/lib/tickets";
	import { GROUPS } from "./CountriesFilter";

	let expanded = false;
	let selectedGroups = new Set<string>();

	function toggleMenu() {
		expanded = !expanded;
	}

	function toggleGroup(group: { name: string; flag: string; list: string[] }) {
		if (selectedGroups.has(group.name)) {
			selectedGroups.delete(group.name);
		} else {
			selectedGroups.add(group.name);
		}
		selectedGroups = new Set(selectedGroups);
		const selectedCountries =
			selectedGroups.size > 0
				? GROUPS.filter((g) => selectedGroups.has(g.name)).flatMap((g) => g.list)
				: [];
		$countriesFilter = selectedCountries;
	}
</script>

<nav class="country-menu-wrapper">
	<GlobeHemisphereWest />
	<button onclick={toggleMenu} class="country-menu-header">
		Groups {#if expanded}
			<CaretDown />
		{:else}
			<CaretRight />
		{/if}
	</button>
	{#if expanded}
		<div class="country-menu" transition:fade={{ duration: 100 }}>
			<div class="country-link-list">
				{#each GROUPS as group}
					<button class="country-menu-button" onclick={() => toggleGroup(group)}>
						<CountryFlag code={group.flag} name={group.name} height={1.2 * 16} />
						{group.name}
						<span style="flex-grow: 1;"></span>
						{#if selectedGroups.has(group.name)}
							<Check />
						{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}
</nav>

<style lang="scss">
	.country-menu-wrapper {
		position: relative;
		background: var(--bg-medium);
		border-radius: 0.5rem;
		box-shadow: 0px 4px 6px rgba(0, 0, 0, 0.1);
		padding: 0.5rem;
		display: flex;
		justify-content: center;
		align-items: center;
		gap: 0.5rem;
	}

	.country-menu {
		position: absolute;
		background: var(--bg-medium);
		border-radius: 0.5rem;
		box-shadow: 0px 4px 6px rgba(0, 0, 0, 0.1);
		width: 14rem;
		top: 2.5rem;
		z-index: 100;
	}

	.country-menu-header {
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.country-link-list {
		display: flex;
		flex-direction: column;
	}

	.country-menu-button {
		background: none;
		border: none;
		color: var(--text);
		padding: 0.75rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 1rem;
		&:hover {
			background-color: var(--bg-light);
		}
	}
</style>
