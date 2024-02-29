<script lang="ts">
	import Button from "$/components/button.svelte";
	import EmoteContainer from "$/components/emote-container.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
	import {
		Fire,
		FolderSimple,
		GlobeHemisphereWest,
		GridFour,
		GridNine,
		Trophy,
		Upload,
	} from "phosphor-svelte";
</script>

<div class="nav-bar">
	<div class="tabs">
		<TabLink href="/emotes" title="Trending" responsive>
			<Fire />
			<Fire weight="fill" slot="active" />
		</TabLink>
		<TabLink href="/emotes/top" title="Top" responsive>
			<Trophy />
			<Trophy weight="fill" slot="active" />
		</TabLink>
		<TabLink href="/emotes/global" title="Global" responsive>
			<GlobeHemisphereWest />
			<GlobeHemisphereWest weight="fill" slot="active" />
		</TabLink>
		<TabLink href="/emotes/new" title="New" responsive>
			<Upload />
			<Upload weight="fill" slot="active" />
		</TabLink>
	</div>
	<div class="buttons">
		<Button secondary hideOnMobile>
			<FolderSimple slot="icon" />
			Personal Emotes
		</Button>
		<Button
			secondary={$emotesLayout === Layout.BigGrid}
			on:click={() => ($emotesLayout = Layout.BigGrid)}
		>
			<GridFour slot="icon" />
		</Button>
		<Button
			secondary={$emotesLayout === Layout.SmallGrid}
			on:click={() => ($emotesLayout = Layout.SmallGrid)}
		>
			<GridNine slot="icon" />
		</Button>
	</div>
</div>
<EmoteContainer scrollable layout={$emotesLayout}>
	<slot />
</EmoteContainer>

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
