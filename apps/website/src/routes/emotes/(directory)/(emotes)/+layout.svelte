<script lang="ts">
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { Fire, Plant, Trophy } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import DefaultEmoteSetButton from "$/components/default-emote-set-button.svelte";
	import Select from "$/components/input/select.svelte";
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";
	import { emotesLayout, selectionModeInEmotes } from "$/lib/layout";
	import Toggle from "$/components/input/toggle.svelte";
	import Button from "$/components/input/button.svelte";

	let { children }: { children: Snippet } = $props();

	let trendingMetric = $state<string>("");

	$effect(() => {
		if ($page.url.pathname.startsWith("/emotes/trending")) {
			let newPath = "/emotes/trending";

			if (trendingMetric) {
				newPath += `/${trendingMetric}`;
			}

			goto(newPath);
		}
	});
</script>

<div class="nav-bar">
	<div class="nav-left-group">
		<nav class="tabs">
			<TabLink href="/emotes" title={$t("common.top")} responsive>
				<Trophy />
				{#snippet active()}
					<Trophy weight="fill" />
				{/snippet}
			</TabLink>

			<TabLink
				href={"/emotes/trending" + (trendingMetric ? `/${trendingMetric}` : "")}
				title={$t("common.trending")}
				matcher={(page, href) => (href ? page.url.pathname.startsWith(href) : false)}
				responsive
			>
				<Fire />
				{#snippet active()}
					<Fire weight="fill" />
				{/snippet}
			</TabLink>

			<TabLink href="/emotes/new" title={$t("common.new")} responsive>
				<Plant />
				{#snippet active()}
					<Plant weight="fill" />
				{/snippet}
			</TabLink>

			{#if $page.url.pathname.startsWith("/emotes/trending")}
				<Select
					bind:selected={trendingMetric}
					options={[
						{ label: $t("common.Today"), value: "daily" },
						{ label: $t("common.Weekly"), value: "" },
						{ label: $t("common.Monthly"), value: "monthly" },
					]}
				/>
			{/if}
		</nav>

		<Button secondary onclick={() => selectionModeInEmotes.update((value) => !value)}>
			{$t("labels.select")}
			{#snippet iconRight()}
				<Toggle bind:value={$selectionModeInEmotes} />
			{/snippet}
		</Button>
	</div>

	<div class="buttons">
		<DefaultEmoteSetButton />
		<LayoutButtons bind:value={$emotesLayout} />
	</div>
</div>

{@render children()}

<style lang="scss">
	.nav-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 1rem;
	}

	/* Added this to keep the tabs and the button aligned together */
	.nav-left-group {
		display: flex;
		align-items: center;
		gap: 0.75rem; /* Space between the tab pill and the select button */
	}

	.tabs {
		display: flex;
		align-items: center;
		background-color: var(--bg-light);
		border-radius: 0.5rem;
		gap: 0.3rem;
		padding: 0.3rem;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
