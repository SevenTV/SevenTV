<script lang="ts">
	import Button from "$/components/button.svelte";
	import EmoteContainer from "$/components/emote-container.svelte";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { emotesLayout } from "$/lib/stores";
	import { Fire, FolderSimple, GlobeHemisphereWest, Trophy, Upload } from "phosphor-svelte";
</script>

<div class="nav-bar">
	<nav class="tabs">
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
	</nav>
	<div class="buttons">
		<Button secondary hideOnMobile>
			<FolderSimple slot="icon" />
			Personal Emotes
		</Button>
		<LayoutButtons />
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
