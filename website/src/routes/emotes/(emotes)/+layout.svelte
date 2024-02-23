<script lang="ts">
	import Tabs from "$/components/tabs.svelte";
	import { sideBar } from "$/lib/stores";
	import { ArrowLineRight, FolderSimple, GridFour, GridNine } from "phosphor-svelte";
</script>

<div class="nav-bar">
	<div class="buttons">
		{#if !$sideBar}
			<button class="button square expand" on:click={() => ($sideBar = true)}>
				<ArrowLineRight />
			</button>
		{/if}
		<Tabs
			tabs={[
				{ name: "Top", pathname: "/emotes/top" },
				{ name: "Trending", pathname: "/emotes" },
			]}
		/>
	</div>
	<div class="buttons">
		<button class="button secondary icon-left hide-on-mobile">
			<FolderSimple />
			Personal Emotes
		</button>
		<button class="button square">
			<GridFour />
		</button>
		<button class="button square">
			<GridNine />
		</button>
	</div>
</div>
<slot />

<style lang="scss">
	.nav-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
