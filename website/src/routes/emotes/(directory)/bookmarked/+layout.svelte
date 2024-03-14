<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import Button from "$/components/input/button.svelte";
	import { FolderSimple, Smiley, User } from "phosphor-svelte";
	import { defaultEmoteSetDialogMode } from "$/lib/stores";
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import { t } from "svelte-i18n";
</script>

<div class="nav-bar">
	<nav class="tabs">
		<TabLink href="/emotes/bookmarked" title={$t("common.emotes", { values: { count: 2 } })}>
			<Smiley />
			<Smiley weight="fill" slot="active" />
		</TabLink>
		<TabLink href="/emotes/bookmarked/sets" title={$t("common.emote_sets", { values: { count: 2 } })}>
			<FolderSimple />
			<FolderSimple weight="fill" slot="active" />
		</TabLink>
		<TabLink href="/emotes/bookmarked/users" title={$t("common.users", { values: { count: 2 } })}>
			<User />
			<User weight="fill" slot="active" />
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
