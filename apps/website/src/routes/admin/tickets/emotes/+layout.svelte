<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { numberFormat } from "$/lib/utils";
	import { adminTicketsLayout } from "$/lib/layout";
	import { t } from "svelte-i18n";
	import Button from "$/components/input/button.svelte";
	import { countriesFilter, refetchRequested } from "$/lib/tickets";
	import {
		Checks,
		Eye,
		Star,
		ListBullets,
		GridFour,
		MagnifyingGlass,
		Slideshow,
		ArrowClockwise,
	} from "phosphor-svelte";
	import EmoteTicketsButtonOptions from "$/components/admin/emote-tickets-button-options.svelte";
	import { browser } from "$app/environment";
	import TextInput from "$/components/input/text-input.svelte";
	import { totalPublicRequests, totalPersonalRequests, galleryTicketMode } from "$/lib/tickets";
	import CountryFilterDropdown from "$/components/admin/country-filter-dropdown.svelte";

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

	let countriesFilterValue = $state("");
</script>

<nav class="nav-bar">
	<div class="tabs">
		<TabLink
			title="{$t('pages.admin.tickets.emotes.public_listing')} ({numberFormat().format(
				$totalPublicRequests,
			)})"
			href="/admin/tickets/emotes"
			responsive
		>
			<Eye />
			{#snippet active()}
				<Eye weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.emotes.personal_use')} ({numberFormat().format(
				$totalPersonalRequests,
			)})"
			href="/admin/tickets/emotes/personal-use"
			responsive
		>
			<Star />
			{#snippet active()}
				<Star weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.emotes.resolved')} ({$t('common.not-implemented')})"
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
		<Button
			style="background: var(--bg-medium);margin-right: 15px;"
			onclick={() => {
				$refetchRequested = true;
			}}
		>
			{#snippet icon()}
				<ArrowClockwise />
			{/snippet}
			{$t("labels.refetch")}
		</Button>
		<TextInput
			placeholder={$t("labels.search_country")}
			style="flex: 0 1 20rem; margin-right: 15px;"
			bind:value={countriesFilterValue}
			oninput={() => {
				$countriesFilter =
					countriesFilterValue
						.split(",")
						.map((v) => v.trim())
						.filter((v) => v !== "") || [];
			}}
		>
			{#snippet icon()}
				<MagnifyingGlass />
			{/snippet}
		</TextInput>
		{#if $adminTicketsLayout === "big-grid"}
			<EmoteTicketsButtonOptions bind:buttonOptions />
		{/if}
		<div style="margin-right: 15px;">
			<CountryFilterDropdown />
		</div>
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
		<Button
			onclick={() => {
				$galleryTicketMode = true;
			}}
		>
			{#snippet icon()}
				<Slideshow />
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
