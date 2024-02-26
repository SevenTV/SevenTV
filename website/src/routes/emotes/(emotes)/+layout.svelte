<script lang="ts">
	import Button from "$/components/button.svelte";
	import Tabs from "$/components/tabs.svelte";
	import { sideBar } from "$/lib/stores";
	import { ArrowLineRight, FolderSimple, GridFour, GridNine } from "phosphor-svelte";
</script>

<div class="nav-bar">
	<div class="buttons">
		{#if !$sideBar}
			<Button on:click={() => ($sideBar = true)}>
				<ArrowLineRight slot="icon" />
			</Button>
		{/if}
		<Tabs
			tabs={[
				{ name: "Top", pathname: "/emotes/top" },
				{ name: "Trending", pathname: "/emotes" },
			]}
		/>
	</div>
	<div class="buttons">
		<Button primary hideOnMobile>
			<FolderSimple slot="icon" />
			Personal Emotes
		</Button>
		<Button>
			<GridFour slot="icon" />
		</Button>
		<Button>
			<GridNine slot="icon" />
		</Button>
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
