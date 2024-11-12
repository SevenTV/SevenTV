<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import {
		Checks,
		Eye,
		Star,
		Check,
		EyeSlash,
		Trash,
		ListBullets,
		GridFour,
	} from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { adminTicketsLayout } from "$/lib/layout";
	import EmoteTicketsTable from "./emote-tickets-table.svelte";
	import EmoteTicket from "./emote-ticket.svelte";
	import { t } from "svelte-i18n";
	import { numberFormat } from "$/lib/utils";
	import EmoteTicketDialog from "../dialogs/emote-ticket-dialog.svelte";
	import { type DialogMode } from "../dialogs/dialog.svelte";
	import EmoteTicketsButtonOptions from "./emote-tickets-button-options.svelte";
	import { browser } from "$app/environment";

	let selectedMap: boolean[] = $state(Array(20).fill(false));

	let anySelected = $derived(selectedMap.some((v) => v));

	let emoteTicketDialogMode: DialogMode = $state("hidden");

	function showEmoteTicketDialog() {
		emoteTicketDialogMode = "shown";
	}

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

	$effect(() => {
		if (buttonOptions && browser) {
			window.localStorage.setItem("emoteTicketsButtonOptions", JSON.stringify(buttonOptions));
		}
	});
</script>

<EmoteTicketDialog bind:mode={emoteTicketDialogMode} />
<nav class="nav-bar">
	<div class="tabs">
		<TabLink
			title="{$t('pages.admin.tickets.emotes.public_listing')} ({numberFormat().format(9932)})"
			href="/admin/tickets/emotes"
			responsive
		>
			<Eye />
			{#snippet active()}
				<Eye weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.emotes.personal_use')} ({numberFormat().format(412)})"
			href="/admin/tickets/emotes/personal-use"
			responsive
		>
			<Star />
			{#snippet active()}
				<Star weight="fill" />
			{/snippet}
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.emotes.resolved')} ({numberFormat().format(100_000)})"
			href="/admin/tickets/emotes/resolved"
			responsive
		>
			<Checks />
			{#snippet active()}
				<Checks weight="fill" />
			{/snippet}
		</TabLink>
	</div>
	{#if anySelected}
		<div class="buttons">
			<Button>
				{#snippet icon()}
					<Trash color="var(--danger)" />
				{/snippet}
			</Button>
			<Button>
				{#snippet icon()}
					<EyeSlash color="#e0823d" />
				{/snippet}
			</Button>
			<Button>
				{#snippet icon()}
					<Check color="#57ab5a" />
				{/snippet}
			</Button>
		</div>
	{/if}
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
<div class="scroll">
	{#if $adminTicketsLayout === "list"}
		<EmoteTicketsTable bind:selectedMap bind:buttonOptions onclick={showEmoteTicketDialog} />
	{:else}
		<div class="tickets-grid">
			{#each Array(selectedMap.length) as _}
				<EmoteTicket bind:buttonOptions onclick={showEmoteTicketDialog} />
			{/each}
		</div>
	{/if}
</div>

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

	.layout.buttons {
		margin-left: auto;
	}

	.scroll {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;
	}

	.tickets-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(30rem, 1fr));
		gap: 1rem;
	}

	@media screen and (max-width: 960px) {
		.tickets-grid {
			grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		}
	}
</style>
