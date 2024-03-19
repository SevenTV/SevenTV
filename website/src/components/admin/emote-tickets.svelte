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
		GridNine,
	} from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { Layout, adminTicketsLayout } from "$/lib/stores";
	import EmoteTicketsTable from "./emote-tickets-table.svelte";
	import EmoteTicket from "./emote-ticket.svelte";
	import { t } from "svelte-i18n";
	import { numberFormat } from "$/lib/utils";
	import EmoteTicketDialog from "../dialogs/emote-ticket-dialog.svelte";
	import { DialogMode } from "../dialogs/dialog.svelte";

	let selectedMap: boolean[] = Array(20).fill(false);

	$: anySelected = selectedMap.some((v) => v);

	let emoteTicketDialogMode = DialogMode.Hidden;

	function showEmoteTicketDialog() {
		emoteTicketDialogMode = DialogMode.Shown;
	}
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
			<Eye weight="fill" slot="active" />
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.emotes.personal_use')} ({numberFormat().format(412)})"
			href="/admin/tickets/emotes/personal-use"
			responsive
		>
			<Star />
			<Star weight="fill" slot="active" />
		</TabLink>
		<TabLink
			title="{$t('pages.admin.tickets.emotes.resolved')} ({numberFormat().format(100_000)})"
			href="/admin/tickets/emotes/resolved"
			responsive
		>
			<Checks />
			<Checks weight="fill" slot="active" />
		</TabLink>
	</div>
	{#if anySelected}
		<div class="buttons">
			<Button>
				<Trash slot="icon" color="var(--danger)" />
			</Button>
			<Button>
				<EyeSlash slot="icon" color="#e0823d" />
			</Button>
			<Button>
				<Check slot="icon" color="#57ab5a" />
			</Button>
		</div>
	{/if}
	<div class="buttons layout">
		<Button
			secondary={$adminTicketsLayout === Layout.List}
			on:click={() => ($adminTicketsLayout = Layout.List)}
		>
			<ListBullets slot="icon" />
		</Button>
		<Button
			secondary={$adminTicketsLayout === Layout.SmallGrid}
			on:click={() => ($adminTicketsLayout = Layout.SmallGrid)}
		>
			<GridNine slot="icon" />
		</Button>
	</div>
</nav>
<div class="scroll">
	{#if $adminTicketsLayout === Layout.List}
		<EmoteTicketsTable bind:selectedMap on:click={showEmoteTicketDialog} />
	{:else}
		<div class="tickets-grid">
			{#each Array(selectedMap.length) as _}
				<EmoteTicket on:click={showEmoteTicketDialog} />
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
</style>
