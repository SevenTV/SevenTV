<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import { Check, EyeSlash, Trash } from "phosphor-svelte";
	import { adminTicketsLayout } from "$/lib/layout";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { browser } from "$app/environment";
	import { updateFlags } from "$/lib/emoteMutations";
	import type { Emote, Report, EmotePartial } from "$/gql/graphql";
	import ReportsTicketsTable from "./reports-tickets-table.svelte";
	import { Ulid } from "id128";
	import moment from "moment";
	import ReportTicket from "./report-ticket.svelte";
	import ReportTicketPreview from "./report-ticket-preview.svelte";
	import type { DialogMode } from "../dialogs/dialog.svelte";
	export interface ModRequestsTicket {
		message: Report;
		emote: Emote;
		isActioned: boolean;
	}

	let isExpandedGrid = $state(false);
	let loadMoreTrigger = $state<Element | null>(null);
	let tickets = $state<ModRequestsTicket[]>([]);
	let selectedReport = $state<ModRequestsTicket | null>(null);
	let reportEmoteDialogMode: DialogMode = $state("hidden");
	let currentPage = 0;
	const pageSize = 25;
	let anySelected = false;
	interface Props {
		selectedMap: Report[];
	}
	let { selectedMap = $bindable() }: Props = $props();

	const addMore = () => {
		loadEmotes();
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
		const emoteIdsToLoad = selectedMap.slice(start, end).map((v) => v.target_id);
		if (emoteIdsToLoad.length <= 1) return;

		let emotesPartialByID = await queryGetEmotesById(emoteIdsToLoad);
		let emoteById: Emote[] = emotesPartialByID.map((emotePartial) => {
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
					display_name: emotePartial.owner.display_name,
					id: emotePartial.owner.id,
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
			...emoteById.map((emote) => {
				let message = selectedMap.find((v) => v.target_id === emote.id)!;
				let parsedId = Ulid.fromCanonicalTrusted(message.id);
				let date = moment(parsedId.time);
				return {
					message: {
						...message,
						date,
					},
					emote,
					isActioned: false,
				};
			}),
		];
		currentPage++;
	}

	function showReportTicketDialog(ticket: ModRequestsTicket) {
		selectedReport = ticket;
		reportEmoteDialogMode = "shown";
	}
	function showDeleteEmoteDialog() {}
	function showMergeEmoteDialog() {}

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
			if (selectedMap) {
				loadEmotes();
				tickets = [];
				currentPage = 0;
			}
			window.localStorage.setItem("emoteTicketsButtonOptions", JSON.stringify(buttonOptions));
		}
		if (loadMoreTrigger && currentPage == 0) {
			observer.observe(loadMoreTrigger);
		}
	});
</script>

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

{#if selectedReport}
	<ReportTicketPreview bind:mode={reportEmoteDialogMode} bind:ticket={selectedReport} />
{/if}

<div class="scroll">
	{#if $adminTicketsLayout === "list"}
		<ReportsTicketsTable
			{tickets}
			bind:buttonOptions
			onclick={(ticket: ModRequestsTicket) => showReportTicketDialog(ticket)}
			loademotes={() => loadEmotes()}
			ondelete={() => showDeleteEmoteDialog()}
			onmerge={() => showMergeEmoteDialog()}
		/>
	{:else}
		<div class="tickets-grid {isExpandedGrid ? 'expanded' : ''}">
			{#each tickets as ticket (ticket.emote.id)}
				<ReportTicket
					{ticket}
					bind:buttonOptions
					onrevertticketaction={() => RevertTicketAction(ticket)}
					onaction={(action) => onTicketAction(ticket, action)}
					onclick={() => showReportTicketDialog(ticket)}
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

	@media screen and (max-width: 960px) {
		.tickets-grid {
			grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		}
	}
</style>
