<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { discoverFollowingLayout } from "$/store/layout";
	import { ListDashes, GridFour, Pulse, Upload } from "phosphor-svelte";
	import type { Snippet } from "svelte";
	import { t } from "svelte-i18n";

	let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
	<title>Following - {$t("page_titles.suffix")}</title>
</svelte:head>

<div class="nav">
	<div class="tabs">
		<TabLink title={$t("pages.discover.uploads")} href="/discover/following">
			<Upload />
			{#snippet active()}
				<Upload weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink title={$t("common.activity")} href="/discover/following/activity">
			<Pulse />
			{#snippet active()}
				<Pulse weight="fill" />
			{/snippet}
		</TabLink>
	</div>
	<div class="buttons">
		<Button
			secondary={$discoverFollowingLayout === "list"}
			onclick={() => ($discoverFollowingLayout = "list")}
		>
			{#snippet icon()}
				<ListDashes />
			{/snippet}
		</Button>
		<Button
			secondary={$discoverFollowingLayout === "big-grid"}
			onclick={() => ($discoverFollowingLayout = "big-grid")}
		>
			{#snippet icon()}
				<GridFour />
			{/snippet}
		</Button>
	</div>
</div>
{@render children()}

<style lang="scss">
	.nav {
		display: flex;
		justify-content: space-between;
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
