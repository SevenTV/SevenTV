<script lang="ts">
	import Button from "$/components/button.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { sideBar } from "$/lib/stores";
	import { ArrowLineRight, Fire, FolderSimple, GlobeHemisphereWest, GridFour, GridNine, Trophy, Upload } from "phosphor-svelte";
</script>

<div class="nav-bar">
	<div class="buttons">
		{#if !$sideBar}
			<Button on:click={() => ($sideBar = true)}>
				<ArrowLineRight slot="icon" />
			</Button>
		{/if}
		<div class="tabs">
			<TabLink href="/emotes" title="Trending">
				<Fire />
				<Fire weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/top" title="Top">
				<Trophy />
				<Trophy weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/global" title="Global">
				<GlobeHemisphereWest />
				<GlobeHemisphereWest weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/new" title="New">
				<Upload />
				<Upload weight="fill" slot="active" />
			</TabLink>
		</div>
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

	.tabs {
		display: flex;
		align-items: center;
		background-color: var(--bg-light);
		border-radius: 0.5rem;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
