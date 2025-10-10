<script lang="ts">
	import moment from "moment/min/moment-with-locales";
	import Date from "../date.svelte";
	import { t } from "svelte-i18n";
	import EmoteTicketsTableActions from "./emote-tickets-table-actions.svelte";
	import EmoteTicketsTableActionsHeader from "./emote-tickets-table-actions-header.svelte";
	import { browser } from "$app/environment";
	import type { ButtonOptions } from "./emote-ticket.svelte";
	import { updateFlags } from "$/lib/emoteMutations";
	import ResponsiveImage from "../responsive-image.svelte";
	import type { Emote, Report } from "$/gql/graphql";
	import Button from "../input/button.svelte";
	import { ArrowCounterClockwise, EnvelopeOpen, LockSimple, Spinner } from "phosphor-svelte";
	import { expandedTickets } from "$/lib/tickets";
	interface ModRequestsTicket {
		message: Report;
		emote: Emote;
		isActioned: boolean;
	}

	interface Props {
		buttonOptions: ButtonOptions;
		tickets: ModRequestsTicket[];
		onclick: (ticket: ModRequestsTicket) => void;
		loademotes: () => void;
		ondelete: (emote: Emote) => void;
		onmerge: (emote: Emote) => void;
	}

	let {
		buttonOptions = $bindable(),
		tickets,
		onclick,
		loademotes,
		ondelete,
		onmerge,
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
			let aValue: string | moment.Moment | undefined;
			let bValue: string | moment.Moment | undefined;
			switch (key) {
				case "reportedBy":
					aValue = a.message.actor.display_name;
					bValue = b.message.actor.display_name;
					break;
				case "target":
					aValue = a.emote.defaultName;
					bValue = b.emote.defaultName;
					break;
				case "description":
					aValue = a.message.subject;
					bValue = b.message.subject;
					break;
				case "assignedTo":
					aValue = a.message.assignees[0]?.display_name ?? "Unassigned";
					bValue = b.message.assignees[0]?.display_name ?? "Unassigned";
					break;
				case "status":
					aValue = a.message.status;
					bValue = b.message.status;
					break;
				case "ticketId":
				case "date":
					aValue = moment(a.message.created_at);
					bValue = moment(b.message.created_at);
					break;
				default:
					return 0;
			}
			if (typeof aValue === "string" && typeof bValue === "string") {
				return sortOrder === "asc" ? aValue.localeCompare(bValue) : bValue.localeCompare(aValue);
			}
			if (moment.isMoment(aValue) && moment.isMoment(bValue)) {
				return sortOrder === "asc" ? aValue.diff(bValue) : bValue.diff(aValue);
			}
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
				await ondelete(ticket.emote);
				success = true;
				break;
			case "merge":
				await onmerge(ticket.emote);
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
			}
		}

		return success;
	}

	async function RevertTicketAction(ticket: ModRequestsTicket) {
		const revertedFlags = {
			...ticket.emote.flags,
			publicListed: false,
			approvedPersonal: false,
			deniedPersonal: false,
		};
		await updateFlags(ticket.emote.id, revertedFlags);
		ticket.isActioned = false;
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
			{#if actionsPosition === "left"}
				<EmoteTicketsTableActionsHeader bind:buttonOptions bind:actionsPosition />
			{/if}
			<th class="sortable" onclick={() => sortTickets("reportedBy")}>
				{$t("pages.admin.tickets.reports_table.reported_by")}</th
			>
			<th class="sortable" onclick={() => sortTickets("target")}
				>{$t("pages.admin.tickets.reports_table.target")}</th
			>
			<th class="sortable" onclick={() => sortTickets("description")}>
				{$t("pages.admin.tickets.reports_table.description")}
			</th>
			<th class="sortable" onclick={() => sortTickets("assignedTo")}>
				{$t("pages.admin.tickets.reports_table.assigned_to")}
			</th>
			<th class="sortable" onclick={() => sortTickets("status")}>
				{$t("pages.admin.tickets.reports_table.status")}
			</th>
			<th class="sortable" onclick={() => sortTickets("ticketId")}
				>{$t("pages.admin.tickets.reports_table.ticket_id")}</th
			>
			<th class="sortable" onclick={() => sortTickets("date")}
				>{$t("pages.admin.tickets.reports_table.date")}</th
			>

			{#if actionsPosition === "right"}
				<EmoteTicketsTableActionsHeader bind:buttonOptions bind:actionsPosition />
			{/if}
		</tr>
	</thead>
	<tbody>
		{#each tickets as { message, emote, isActioned }, i}
			<tr class:data-row={true} class:actioned={isActioned} onclick={() => onclick(tickets[i])}>
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
					<a href="/users/{message.actor.id}" class="reporter" title={message.actor.display_name}>
						{message.actor.display_name}
					</a>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<div class="emote">
						<div class="emote-preivew">
							<ResponsiveImage images={emote.images} />
						</div>
						<a href="/emotes/{emote.id}" class="emote-name" title={emote.defaultName}>
							{emote.defaultName}
						</a>
					</div>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<h4>{message.subject}</h4>
					<div class="description grey">
						<p>{message.body}</p>
					</div>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<div class="assigned-user">
						{#if message.assignees.length == 0}
							<p class="grey">No assignees</p>
						{:else}
							<img
								class="avatar"
								src={message.assignees[0].avatar_url}
								alt={message.assignees[0].display_name}
							/>
							<p>{message.assignees[0].display_name}</p>
						{/if}
					</div>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					{#if message.status === "OPEN"}
						<div class="status" style="background-color: #171f17; color: #57ab5a">
							<EnvelopeOpen />
							Open
						</div>
					{:else if message.status === "ASSIGNED"}
						<div class="status" style="background-color: #161d26; color: #529bf5">
							<Spinner />Pending
						</div>
					{:else if message.status === "CLOSED"}
						<div class="status" style="background-color: #261918; color: #f47068">
							<LockSimple />Closed
						</div>
					{/if}
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<div class="ticket-id grey">
						{message.id}
					</div>
				</td>
				<td class:expanded-tickets={$expandedTickets}>
					<div class="date">
						<Date date={moment(message.created_at)} />
					</div>
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
						title="Revert Action"
						onclick={async (event) => {
							event.stopPropagation();
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
	.sortable {
		cursor: pointer;
	}
	.actioned {
		background-color: rgba(12, 12, 12, 0.8);
		opacity: 0.5;
	}

	.assigned-user {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 8px;
	}

	.status {
		height: 2rem;
		width: 100%;
		border-radius: 10%;
		display: flex;
		justify-content: center;
		align-items: center;
		padding: 0.5rem; /* Added padding for space */
	}

	.ticket-id {
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

	.avatar {
		width: 2rem;
		height: 2rem;
		border-radius: 50%;
	}

	.sortable:hover {
		background-color: #0c0c0c;
	}
	.data-row {
		cursor: pointer;
	}

	.emote {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 8px;
	}

	.grey {
		color: var(--text-light);
	}

	.description {
		display: flex;
		flex-direction: column;
		gap: 4px;
		white-space: nowrap !important;
		overflow: hidden !important;
		text-overflow: ellipsis !important;
		max-width: 42ch !important;
		text-align: left;
	}

	.emote-preivew {
		display: flex;
		align-items: center;

		> :global(picture) {
			height: 2rem;
			width: 2rem;
			flex-grow: 1;
			border-radius: 50%;
		}

		> :global(picture > img) {
			height: 100%;
			width: 100%;
			object-fit: contain;
			border-radius: 15%;
		}
	}

	.reporter {
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
		gap: 0.5rem;
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

	.expanded-tickets {
		padding: 2.5rem 0.75rem !important;
		font-size: 1rem !important;
	}
</style>
