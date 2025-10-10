<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { numberFormat } from "$/lib/utils";
	import { adminTicketsLayout } from "$/lib/layout";
	import { t } from "svelte-i18n";
	import Button from "$/components/input/button.svelte";
	import { Checks, Eye, Star, ListBullets, GridFour } from "phosphor-svelte";
	import EmoteTicketsButtonOptions from "$/components/admin/emote-tickets-button-options.svelte";
	import { browser } from "$app/environment";
	import { totalReportRequests } from "$/lib/tickets";

	const defaultButtonOptions = {
		merge: true,
		delete: true,
		unlist: true,
		approve: true,
	};

	function loadButtonOptions(): {
		merge: boolean;
		delete: boolean;
		unlist: boolean;
		approve: boolean;
	} | null {
		if (!browser) return null;
		const options = window.localStorage.getItem("emoteTicketsButtonOptions");
		return options && JSON.parse(options);
	}

	let buttonOptions = $state(loadButtonOptions() || defaultButtonOptions);

	let { children } = $props();
</script>

<nav class="nav-bar">
	<div class="tabs">
		<TabLink
			title="{$t('pages.admin.tickets.reports_options.all')} ({numberFormat().format(
				$totalReportRequests,
			)})"
			responsive
			href="/admin/tickets/reports"
		>
			<Eye />
			{#snippet active()}
				<Eye weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			title={$t("pages.admin.tickets.reports_options.assigned")}
			responsive
			href="javascript:void(0)"
		>
			<Star />
			{#snippet active()}
				<Star weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.reports_options.closed')} ({$t('common.not-implemented')})"
			href="javascript:void(0)"
			responsive
		>
			<Checks />
			{#snippet active()}
				<Checks weight="fill" />
				<span>{$t("common.not-implemented")}</span>
			{/snippet}
		</TabLink>
	</div>
	<div class="buttons layout">
		{#if $adminTicketsLayout === "big-grid"}
			<EmoteTicketsButtonOptions bind:buttonOptions />
		{/if}
		<Button
			secondary={$adminTicketsLayout === "list"}
			onclick={() => ($adminTicketsLayout = "list")}
		>
			{#snippet icon()}
				<ListBullets />
			{/snippet}
		</Button>
		<Button
			secondary={$adminTicketsLayout === "big-grid"}
			onclick={() => ($adminTicketsLayout = "big-grid")}
		>
			{#snippet icon()}
				<GridFour />
			{/snippet}
		</Button>
	</div>
</nav>
{@render children?.()}

<style lang="scss">
	.nav-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.tabs {
		display: flex;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}

	.buttons {
		display: flex;
		align-items: center;
	}
</style>
