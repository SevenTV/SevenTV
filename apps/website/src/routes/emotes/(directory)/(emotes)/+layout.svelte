<script lang="ts">
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { Fire, Plant, Trophy, Upload } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import DefaultEmoteSetButton from "$/components/default-emote-set-button.svelte";
	import Select from "$/components/input/select.svelte";
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";

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
	<nav class="tabs">
		<TabLink href="/emotes" title={$t("common.top")} responsive>
			<Trophy />
			{#snippet active()}
				<Trophy weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			href="/emotes/trending"
			title={$t("common.trending")}
			matcher={(page, href) => (href ? page.url.pathname.startsWith(href) : false)}
			responsive
		>
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
			<Plant />
			{#snippet active()}
				<Plant weight="fill" />
			{/snippet}
		</TabLink>
		{#if $page.url.pathname.startsWith("/emotes/trending")}
			<Select
				bind:selected={trendingMetric}
				options={[
					{ label: "Today", value: "daily" },
					{ label: "Weekly", value: "" },
					{ label: "Monthly", value: "monthly" },
				]}
			/>
		{/if}
	</nav>
	<div class="buttons">
		<DefaultEmoteSetButton />
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
