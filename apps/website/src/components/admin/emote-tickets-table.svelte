<script lang="ts">
	import Flags from "$/components/flags.svelte";
	import moment from "moment/min/moment-with-locales";
	import Date from "../date.svelte";
	import CountryFlag from "../country-flag.svelte";
	import { t } from "svelte-i18n";
	import EmoteTicketsTableActions from "./emote-tickets-table-actions.svelte";
	import EmoteTicketsTableActionsHeader from "./emote-tickets-table-actions-header.svelte";
	import { browser } from "$app/environment";
	import type { ButtonOptions } from "./emote-ticket.svelte";
	import type { ModRequestsTicket } from "./emote-tickets.svelte";
	import { updateFlags } from "$/lib/emoteMutations";
	import ResponsiveImage from "../responsive-image.svelte";
	import type { Emote } from "$/gql/graphql";
	import Button from "../input/button.svelte";
	import { expandedTickets } from "$/lib/tickets";
	import { ArrowCounterClockwise } from "phosphor-svelte";
	import type { CountryCode } from "./CountriesFilter";

	interface Props {
		buttonOptions: ButtonOptions;
		tickets: ModRequestsTicket[];
		onclick: (index: number) => void;
		loademotes: () => void;
		ondelete: (ticket: ModRequestsTicket) => void;
		onmerge: (ticket: ModRequestsTicket) => void;
		onnotification: (message: string) => void;
	}

	let {
		buttonOptions = $bindable(),
		tickets = $bindable(),
		onclick,
		loademotes,
		ondelete,
		onmerge,
		onnotification,
	}: Props = $props();

	let sortKey = "";
	let sortOrder = "asc";
	function sortTickets(key: string) {
		if (sortKey === key) {
			sortOrder = sortOrder === "asc" ? "desc" : "asc";
		} else {
			sortKey = key;
			sortOrder = "asc";
		}

		tickets.sort((a, b) => {
			let aValue, bValue;
			switch (key) {
				case "emote":
					aValue = a.emote.defaultName;
					bValue = b.emote.defaultName;
					break;
				case "emoteName":
					aValue = a.emote.defaultName;
					bValue = b.emote.defaultName;
					break;
				case "uploader":
					aValue = a.emote.owner?.mainConnection?.platformUsername;
					bValue = b.emote.owner?.mainConnection?.platformUsername;
					break;
				case "country":
					aValue = a.message.actor_country_name;
					bValue = b.message.actor_country_name;
					break;
				case "tags":
					aValue = a.emote.tags.join(", ");
					bValue = b.emote.tags.join(", ");
					break;
				case "Date":
					aValue = moment(a.message.created_at);
					bValue = moment(b.message.created_at);
					break;
				default:
					return 0;
			}
			if (typeof aValue === "string" && typeof bValue === "string") {
				return sortOrder === "asc" ? aValue.localeCompare(bValue) : bValue.localeCompare(aValue);
			}

			if ((aValue ?? "") < (bValue ?? "")) return sortOrder === "asc" ? -1 : 1;
			if ((aValue ?? "") > (bValue ?? "")) return sortOrder === "asc" ? 1 : -1;
			return 0;
		});
	}

	// The commented code below is to handle mass approve/deny ( Currently commented out as it needs a rework )
	// let allSelected = $derived(localSelectedMap.every((v) => v));
	// let anySelected = $derived(localSelectedMap.some((v) => v));

	// function selectAllClick() {
	// 	localSelectedMap = Array(localSelectedMap.length).fill(!allSelected);
	// }

	let loadMoreTrigger: Element;

	const addMore = () => {
		loademotes();
	};

	const observer = new IntersectionObserver(
		(entries) => {
			if (entries[0].isIntersecting) {
				addMore();
			}
		},
		{
			rootMargin: "100px",
		},
	);

	$effect(() => {
		if (loadMoreTrigger) {
			observer.observe(loadMoreTrigger);
		}
	});

	async function onTicketAction(ticket: ModRequestsTicket, action: string) {
		const newFlags: Partial<Emote["flags"]> = {};
		let success = false;

		switch (action) {
			case "approve":
				newFlags.publicListed = true;
				break;
			case "unlist":
				newFlags.publicListed = false;
				break;
			case "delete":
				await ondelete(ticket);
				success = true;
				break;
			case "merge":
				await onmerge(ticket);
				success = true;
				break;
			default:
				return false;
		}

		if (!success) {
			const updatedEmote = await updateFlags(ticket.emote.id, newFlags);
			if (updatedEmote) {
				ticket.isActioned = true;
				success = true;
				onnotification(`${action} Success!`);
			}
			onnotification(`Couldn't perform ${action} successfully!`);
		}

		return success;
	}

	async function RevertTicketAction(emote: ModRequestsTicket) {
		const revertedFlags = {
			...emote.emote.flags,
			publicListed: false,
			approvedPersonal: false,
			deniedPersonal: false,
		};
		await updateFlags(emote.emote.id, revertedFlags);
		emote.isActioned = false;
	}

	function loadActionsPosition(): "left" | "right" | null {
		if (!browser) return null;
		const position = window.localStorage.getItem("emoteTicketsActionsPosition");
		return position && JSON.parse(position);
	}

	let actionsPosition: "left" | "right" = $state(loadActionsPosition() || "left");

	$effect(() => {
		if (actionsPosition && browser) {
			window.localStorage.setItem("emoteTicketsActionsPosition", JSON.stringify(actionsPosition));
		}
	});
</script>

<table>
	<thead>
		<tr>
			<!-- same here for the mass approve/deny -->
			<!-- <th class="shrink">
				<Checkbox
					value={allSelected}
					indeterminate={anySelected && !allSelected}
					onclick={selectAllClick}
				/>
			</th> -->
			{#if actionsPosition === "left"}
				<EmoteTicketsTableActionsHeader bind:buttonOptions bind:actionsPosition />
			{/if}
			<th class="sortable" onclick={() => sortTickets("emote")}>
				{$t("common.emotes", {
					values: { count: 1 },
				})}</th
			>
			<th class="sortable" onclick={() => sortTickets("emoteName")}
				>{$t("pages.admin.tickets.emote_table.emote_name")}</th
			>
			<th class="sortable" onclick={() => sortTickets("uploader")}>
				{$t("pages.admin.tickets.emote_table.uploader")}
			</th>
			<th class="sortable" onclick={() => sortTickets("country")}>
				{$t("pages.admin.tickets.emote_table.country")}
			</th>
			<th class="sortable" onclick={() => sortTickets("tags")}>
				{$t("pages.admin.tickets.emote_table.tags_flags")}
			</th>
			<th class="sortable" onclick={() => sortTickets("Date")}> Date </th>

			{#if actionsPosition === "right"}
				<EmoteTicketsTableActionsHeader bind:buttonOptions bind:actionsPosition />
			{/if}
		</tr>
	</thead>
	<tbody>
		{#each tickets as { message, emote, isActioned }, i}
			<tr class:data-row={true} class:actioned={isActioned} onclick={() => onclick(i)}>
				<!-- same here for the mass approve/deny -->
				<!-- <td class="shrink">
					<Checkbox bind:value={localSelectedMap[i]} />
				</td> -->
				{#if actionsPosition === "left" && isActioned}
					<Button
						big
						title="Revert action"
						onclick={async (event) => {
							event.stopPropagation();
							await RevertTicketAction(tickets[i]);
						}}
					>
						{#snippet icon()}
							<ArrowCounterClockwise color="lightblue" />
						{/snippet}
					</Button>
				{/if}
				{#if actionsPosition === "left" && !isActioned}
					<EmoteTicketsTableActions
						onaction={async (action) => {
							await onTicketAction(tickets[i], action);
						}}
						bind:buttonOptions
					/>
				{/if}
				<td class:expanded-tickets={$expandedTickets}>
					<div class:expanded-tickets={$expandedTickets} class="emote-preivew">
						<ResponsiveImage images={emote.images} />
					</div>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<p class:expanded-tickets={$expandedTickets} class="emote-name" title={emote.defaultName}>
						{emote.defaultName}
					</p>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<p class="uploader" title={emote.owner?.mainConnection?.platformUsername}>
						{emote.owner?.mainConnection?.platformUsername}
					</p>
				</td>
				<!-- <td>{numberFormat().format(999)}</td> -->
				<td class:expanded-tickets={$expandedTickets}>
					<CountryFlag
						code={message.actor_country_code as CountryCode}
						name={message.actor_country_name}
						width={35}
					/>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					{#if emote.tags}
						<Flags flags={emote.tags} />
					{/if}
					{#if emote.tags.length === 0}
						<Flags flags={["none"]} />
					{/if}
				</td>
				<!-- <td>
					<div class="mod">
						<div class="placeholder"></div>
						Mod Name
					</div>
				</td> -->
				<td class="date shrink" class:expanded-tickets={$expandedTickets}>
					<Date date={moment(message.created_at)} />
				</td>
				{#if actionsPosition === "right" && !isActioned}
					<EmoteTicketsTableActions
						onaction={async (action) => {
							await onTicketAction(tickets[i], action);
						}}
						bind:buttonOptions
					/>
				{/if}
				{#if actionsPosition === "right" && isActioned}
					<Button
						big
						title="Revert action"
						onclick={async (event) => {
							event.stopPropagation();
							const revertedFlags = {
								...emote.flags,
								publicListed: false,
								approvedPersonal: false,
								deniedPersonal: false,
							};
							await updateFlags(emote.id, revertedFlags);
							tickets[i].isActioned = false;
						}}
					>
						{#snippet icon()}
							<ArrowCounterClockwise color="lightblue" />
						{/snippet}
					</Button>
				{/if}
			</tr>
		{/each}
		<tr bind:this={loadMoreTrigger} class="load-trigger data-row"> </tr>
	</tbody>
</table>

<style lang="scss">
	.expanded-tickets {
		padding: 0.5rem 0.75rem !important;
		font-size: 1rem !important;
	}

	.sortable {
		cursor: pointer;
	}
	.actioned {
		background-color: rgba(12, 12, 12, 0.8);
		opacity: 0.5;
	}

	.sortable:hover {
		background-color: #0c0c0c;
	}
	.data-row {
		cursor: pointer;
	}

	.emote-preivew {
		display: flex;
		align-items: center;

		> :global(picture) {
			height: 2rem;
			width: auto;
			flex-grow: 1;
		}

		> :global(picture > img) {
			height: 100%;
			width: 100%;
			object-fit: contain;
		}
	}

	.expanded-tickets .emote-preivew {
		> :global(picture) {
			height: 8rem !important;
		}
	}

	.uploader {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: var(--text);
		white-space: nowrap !important;
		overflow: hidden !important;
		text-overflow: ellipsis !important;
		display: block !important;
		max-width: 12ch !important;
		text-align: left;
	}

	.emote-name {
		display: flex;
		align-items: center;
		color: var(--text);
		white-space: nowrap !important;
		overflow: hidden !important;
		text-overflow: ellipsis !important;
		display: block !important;
		max-width: 12ch !important;
		text-align: left;
	}

	.date {
		color: var(--text-light);
		white-space: nowrap;
	}
</style>
