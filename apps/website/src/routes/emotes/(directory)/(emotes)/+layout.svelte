<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { defaultEmoteSetDialogMode } from "$/store/layout";
	import { Fire, FolderSimple, Trophy, Upload } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";

	let { children }: { children: Snippet } = $props();
</script>

<div class="nav-bar">
	<nav class="tabs">
		<TabLink href="/emotes" title={$t("common.top")} responsive>
			<Trophy />
			{#snippet active()}
				<Trophy weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink href="/emotes/trending" title={$t("common.trending")} responsive>
			<Fire />
			{#snippet active()}
				<Fire weight="fill" />
			{/snippet}
		</TabLink>
		<!-- <TabLink href="/emotes/global" title={$t("common.emotes", { values: { count: 2 } })} responsive>
			<GlobeHemisphereWest />
			<GlobeHemisphereWest weight="fill" />
		</TabLink> -->
		<TabLink href="/emotes/new" title={$t("common.new")} responsive>
			<Upload />
			{#snippet active()}
				<Upload weight="fill" />
			{/snippet}
		</TabLink>
	</nav>
	<div class="buttons">
		<Button secondary hideOnMobile onclick={() => ($defaultEmoteSetDialogMode = "shown")}>
			{#snippet icon()}
				<FolderSimple />
			{/snippet}
			Personal Emotes
		</Button>
		<LayoutButtons />
	</div>
</div>
{@render children()}

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
