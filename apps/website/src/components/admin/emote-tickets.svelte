<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import { Check, EyeSlash, Trash } from "phosphor-svelte";
	import { adminTicketsLayout } from "$/lib/layout";
	import EmoteTicketsTable from "./emote-tickets-table.svelte";
	import EmoteTicket from "./emote-ticket.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { type DialogMode } from "../dialogs/dialog.svelte";
	import { browser } from "$app/environment";
	import { updateFlags } from "$/lib/emoteMutations";
	import type { Emote, ModRequestMessage, EmotePartial } from "$/gql/graphql";
	import DeleteEmoteDialog from "$/components/dialogs/delete-emote-dialog.svelte";
	import MergeEmoteDialog from "$/components/dialogs/merge-emote-dialog.svelte";
	import EmoteTicketPreivew from "./emote-ticket-preview.svelte";
	import { countriesFilter, galleryTicketMode, refetchRequested } from "$/lib/tickets";
	import { writable } from "svelte/store";

	const defaultButtonOptions = {
		merge: true,
		delete: true,
		unlist: true,
		approve: true,
	};
	let notifications = writable<{ id: number; message: string }[]>([]);
	let notificationId = 0;
	let isExpandedGrid = $state(false);
	let loadMoreTrigger = $state<Element | null>(null);
	let tickets = $state<ModRequestsTicket[]>([]);
	let currentPage = 0;
	const pageSize = 25;
	let anySelected = false;
	let deleteEmoteDialogMode: DialogMode = $state("hidden");
	let mergeEmoteDialogMode: DialogMode = $state("hidden");
	let emoteTicketPreviewDialogMode: DialogMode = $state("hidden");
	let emoteToDelete = $state<Emote>();
	let mergeEmote = $state<Emote>();
	let buttonOptions = $state(loadButtonOptions() || defaultButtonOptions);
	let currentTicketIndex = $state(0);
	let loadedEmoteIds = new Set<string>();

	export interface ModRequestsTicket {
		message: ModRequestMessage;
		emote: Emote;
		isActioned: boolean;
	}
	interface Props {
		selectedMap: ModRequestMessage[];
	}
	let { selectedMap = $bindable() }: Props = $props();

	const addMore = () => {
		loadEmotes();
	};
	function showNotification(message: string) {
		const id = notificationId++;
		notifications.update((n) => [...n, { id, message }]);

		setTimeout(() => {
			notifications.update((n) => n.filter((notif) => notif.id !== id));
		}, 30000);
	}

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

	async function onTicketAction(emote: ModRequestsTicket, action: string) {
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
				showDeleteEmoteDialog(emote);
				break;
			default:
				return false;
		}

		if (!success) {
			const updatedEmote = await updateFlags(emote.emote.id, newFlags);
			if (updatedEmote) {
				emote.isActioned = true;
				success = true;
			}
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

	async function queryGetEmotesById(list: Array<string>) {
		const result = await gqlClient()
			.query(
				graphql(`
					query EmotesByID($list: [Id!]!) {
						emotesByID(list: $list) {
							id
							name
							tags
							animated
							owner {
								id
								display_name
							}
							host {
								url
								files {
									name
									format
									width
									height
									size
								}
							}
							flags
							state
						}
					}
				`),
				{ list },
				{
					url: "https://7tv.io/v3/gql",
				},
			)
			.toPromise();

		if (result.error || !result.data || !result.data.emotesByID) {
			throw result.error;
		}
		return result.data.emotesByID as EmotePartial[];
	}

	async function loadEmotes() {
		const start = currentPage * pageSize;
		const end = start + pageSize;
		const emoteIdsToLoad = selectedMap
			.slice(start, end)
			.map((v) => v.target_id)
			.filter((id) => !loadedEmoteIds.has(id)); // Filter out already loaded emotes

		if (emoteIdsToLoad.length === 0) return;

		let emotesPartialByID = await queryGetEmotesById(emoteIdsToLoad);
		let isActioned = false;
		let emoteById: Emote[] = emotesPartialByID.map((emotePartial) => {
			loadedEmoteIds.add(emotePartial.id); // Mark emote as loaded
			return {
				owner_id: emotePartial.owner.id,
				aspectRatio: 1,
				attribution: [],
				channels: {
					items: [],
					pageCount: 0,
					totalCount: 0,
				},
				defaultName: emotePartial.name,
				deleted: false,
				events: [],
				flags: {
					animated: emotePartial.animated,
					...emotePartial.flags,
				},
				id: emotePartial.id,
				imagesPending: false,
				inEmoteSets: [],
				owner: {
					id: emotePartial.owner.id,
					mainConnection: {
						platformUsername: emotePartial.owner.display_name,
					},
				},
				ownerId: emotePartial.owner.id,
				scores: {
					topAllTime: 0,
					topDaily: 0,
					topMonthly: 0,
					topWeekly: 0,
					trendingDay: 0,
					trendingMonth: 0,
					trendingWeek: 0,
				},
				tags: emotePartial.tags,
				updatedAt: new Date(),
				images: emotePartial.host.files.map((file) => ({
					frameCount: 2,
					mime: "image/" + file.format.toLowerCase(),
					scale: 1,
					url: emotePartial.host.url + "/" + file.name,
					...file,
				})),
			} as unknown as Emote;
		});
		tickets = [
			...tickets,
			...emoteById.map((emote) => ({
				message: selectedMap.find((v) => v.target_id === emote.id)!,
				emote,
				isActioned,
			})),
		];
		currentPage++;
	}
	function showEmoteTicketDialog(emote: Emote) {
		currentTicketIndex = tickets.findIndex((t) => t.emote.id === emote.id);
		emoteTicketPreviewDialogMode = "shown";
	}
	function showDeleteEmoteDialog(ticket: ModRequestsTicket) {
		emoteToDelete = ticket.emote;
		deleteEmoteDialogMode = "shown";
	}
	function showMergeEmoteDialog(ticket: ModRequestsTicket) {
		mergeEmote = ticket.emote;
		mergeEmoteDialogMode = "shown";
	}

	function showGalleryEmoteDialog() {
		emoteTicketPreviewDialogMode = "shown";
	}

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

	async function nextTicket(action: string, ticket: ModRequestsTicket) {
		if (action == "next") {
			if (currentTicketIndex < tickets.length - 1) {
				currentTicketIndex++;
			}
			return;
		}
		const success = await onTicketAction(ticket, action);
		if (success == false) {
			showNotification(`Couldn't perform ${action} successfully!`);
			return;
		} else {
			showNotification(`${action} success!`);
		}
		if (success && currentTicketIndex < tickets.length - 1) {
			currentTicketIndex++;
		}
	}

	async function previousTicket(action: string, ticket: ModRequestsTicket) {
		if (action == "previous") {
			if (currentTicketIndex > 0) {
				currentTicketIndex--;
			}
			return;
		}
		const success = await onTicketAction(ticket, action);
		if (success && currentTicketIndex > 0) {
			currentTicketIndex--;
		}
	}

	$effect(() => {
		if (buttonOptions && browser) {
			if ($refetchRequested == true || $countriesFilter) {
				loadedEmoteIds.clear();
				$refetchRequested = false;
			}
			if (selectedMap) {
				loadEmotes();
				tickets = [];
				currentPage = 0;
				currentTicketIndex = 0;
			}
			window.localStorage.setItem("emoteTicketsButtonOptions", JSON.stringify(buttonOptions));
		}
		if (loadMoreTrigger && currentPage == 0) {
			observer.observe(loadMoreTrigger);
		}
		//Scuffed way to open gallery dialog, But will make it better
		if ($galleryTicketMode == true && emoteTicketPreviewDialogMode == "hidden") {
			showGalleryEmoteDialog();
		} else {
			$galleryTicketMode = false;
		}
	});
</script>

<div class="notification-container">
	{#each $notifications as notification (notification.id)}
		<div class="notification">
			{notification.message}
		</div>
	{/each}
</div>
{#if emoteToDelete}
	<DeleteEmoteDialog bind:mode={deleteEmoteDialogMode} data={emoteToDelete} />
{/if}

{#if mergeEmote}
	<MergeEmoteDialog bind:mode={mergeEmoteDialogMode} data={mergeEmote} />
{/if}

{#if tickets[currentTicketIndex]}
	<EmoteTicketPreivew
		bind:mode={emoteTicketPreviewDialogMode}
		bind:ticket={tickets[currentTicketIndex]}
		bind:emoteIndex={currentTicketIndex}
		bind:ticketsLoaded={tickets.length}
		loademotes={() => loadEmotes()}
		{RevertTicketAction}
		onnext={async (action: string, ticket: ModRequestsTicket) => await nextTicket(action, ticket)}
		onprevious={async (action: string, ticket: ModRequestsTicket) =>
			await previousTicket(action, ticket)}
	/>
{/if}

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

<div class="scroll">
	{#if $adminTicketsLayout === "list"}
		<EmoteTicketsTable
			{tickets}
			bind:buttonOptions
			onclick={(i) => showEmoteTicketDialog(tickets[i].emote)}
			loademotes={() => loadEmotes()}
			ondelete={(ticket) => showDeleteEmoteDialog(ticket)}
			onmerge={(ticket) => showMergeEmoteDialog(ticket)}
			onnotification={(message) => showNotification(message)}
		/>
	{:else}
		<div class="tickets-grid {isExpandedGrid ? 'expanded' : ''}">
			{#each tickets as ticket, index (ticket.emote.id + "-" + index)}
				<EmoteTicket
					{ticket}
					bind:buttonOptions
					onrevertticketaction={() => RevertTicketAction(ticket)}
					onaction={(action) => onTicketAction(ticket, action)}
					onclick={() => showEmoteTicketDialog(ticket.emote)}
				/>
			{/each}
			<div bind:this={loadMoreTrigger} class="load-trigger"></div>
		</div>
	{/if}
</div>

<style lang="scss">
	.tickets-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(30rem, 1fr));
		gap: 1.5rem;

		&.expanded {
			grid-template-columns: 1fr;
		}
	}

	.tickets-grid > * {
		background-color: var(--bg-light);
		padding: 2rem;
		border: 1px solid var(--bg-medium);
		border-radius: 0.75rem;
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
		font-size: 1.25rem;
	}

	.tickets-grid.expanded > * {
		width: 100%;
		padding: 2.5rem;
		font-size: 4.5rem;
	}

	.buttons {
		display: flex;
		align-items: center;
	}

	.scroll {
		overflow: auto;
		scrollbar-gutter: stable;
	}

	.notification-container {
		position: fixed;
		bottom: 1rem;
		right: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		z-index: 1010;
	}

	.notification {
		background-color: #333;
		color: #fff;
		padding: 1rem 1.5rem;
		border-radius: 0.75rem;
		box-shadow: 0 3px 7px rgba(0, 0, 0, 0.3);
		font-size: 1.25rem;
		animation: fade-in-out 99s ease-in-out;
	}

	@keyframes fade-in-out {
		0% {
			opacity: 0;
			transform: translateY(10px);
		}
		10%,
		90% {
			opacity: 1;
			transform: translateY(0);
		}
		100% {
			opacity: 0;
			transform: translateY(10px);
		}
	}
	@media screen and (max-width: 960px) {
		.tickets-grid {
			grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		}
	}
</style>
