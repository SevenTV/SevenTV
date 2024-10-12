<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { defaultEmoteSetDialogMode } from "$/store/layout";
	import { Fire, FolderSimple, Trophy, Upload } from "phosphor-svelte";
	import { t } from "svelte-i18n";
</script>

<div class="nav-bar">
	<nav class="tabs">
		<TabLink href="/emotes" title={$t("common.top")} responsive>
			<Trophy />
			<Trophy weight="fill" slot="active" />
		</TabLink>
		<TabLink href="/emotes/trending" title={$t("common.trending")} responsive>
			<Fire />
			<Fire weight="fill" slot="active" />
		</TabLink>
		<!-- <TabLink href="/emotes/global" title={$t("common.emotes", { values: { count: 2 } })} responsive>
			<GlobeHemisphereWest />
			<GlobeHemisphereWest weight="fill" slot="active" />
		</TabLink> -->
		<TabLink href="/emotes/new" title={$t("common.new")} responsive>
			<Upload />
			<Upload weight="fill" slot="active" />
		</TabLink>
	</nav>
	<div class="buttons">
		<Button secondary hideOnMobile on:click={() => ($defaultEmoteSetDialogMode = DialogMode.Shown)}>
			<FolderSimple slot="icon" />
			Personal Emotes
		</Button>
		<LayoutButtons />
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
