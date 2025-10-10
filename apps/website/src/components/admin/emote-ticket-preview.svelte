<script lang="ts">
	import {
		Check,
		Gif,
		X,
		Smiley,
		User,
		Spinner,
		CaretLeft,
		CaretRight,
		EyeSlash,
		Trash,
		ArrowSquareIn,
		Eye,
		ArrowCounterClockwise,
	} from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "../dialogs/dialog.svelte";
	import { formatSortIndex } from "../responsive-image.svelte";
	import type { EmoteEvent, Image } from "$/gql/graphql";
	import EmoteInfoImage from "../emotes/emote-info-image.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { isSafari } from "$/lib/utils";
	import EmoteEventComponent from "$/components/events/emote-event.svelte";
	import Flags from "../flags.svelte";
	import TabLink from "../tab-link.svelte";
	import type { ModRequestsTicket } from "./emote-tickets.svelte";
	interface Props {
		mode: DialogMode;
		ticket: ModRequestsTicket;
		emoteIndex: number;
		onnext: (action: string, ticket: ModRequestsTicket) => void;
		onprevious: (action: string, ticket: ModRequestsTicket) => void;
		ticketsLoaded: number;
		loademotes: () => void;
		RevertTicketAction: (ticket: ModRequestsTicket) => Promise<void>;
	}

	let {
		mode = $bindable("hidden"),
		ticket = $bindable(),
		emoteIndex = $bindable(),
		onnext,
		onprevious,
		ticketsLoaded = $bindable(),
		loademotes,
		RevertTicketAction,
	}: Props = $props();
	const webhookUrl = localStorage.getItem("webhookUrl") || null;
	let events = $derived(loadEvents(ticket.emote.id));
	let preparedImages = $state<Image[][]>([]);
	let SpoilerDialogMode: DialogMode = $state("hidden");
	let tab: "activity" | "comments" = $state("activity");
	let showRejectDialog: DialogMode = $state("hidden");

	async function loadEvents(id: string) {
		const res = await gqlClient()
			.query(
				graphql(`
					query EmoteEvents($id: Id!) {
						emotes {
							emote(id: $id) {
								events {
									id
									createdAt
									actor {
										id
										mainConnection {
											platformDisplayName
										}
										highestRoleColor {
											hex
										}
									}
									data {
										__typename
										... on EventEmoteDataProcess {
											event
										}
										... on EventEmoteDataChangeName {
											oldName
											newName
										}
										... on EventEmoteDataMerge {
											newEmote {
												id
												defaultName
											}
										}
										... on EventEmoteDataChangeOwner {
											oldOwner {
												id
												mainConnection {
													platformDisplayName
												}
												highestRoleColor {
													hex
												}
											}
											newOwner {
												id
												mainConnection {
													platformDisplayName
												}
												highestRoleColor {
													hex
												}
											}
										}
										... on EventEmoteDataChangeTags {
											oldTags
											newTags
										}
										... on EventEmoteDataChangeFlags {
											oldFlags {
												publicListed
												private
												defaultZeroWidth
												approvedPersonal
												deniedPersonal
											}
											newFlags {
												publicListed
												private
												defaultZeroWidth
												approvedPersonal
												deniedPersonal
											}
										}
									}
								}
							}
						}
					}
				`),
				{ id },
			)
			.toPromise();

		if (res.error || !res.data) {
			throw res.error;
		}

		return res.data.emotes.emote?.events as EmoteEvent[];
	}

	function prepareImages(images: Image[]): Image[][] {
		const safari = isSafari();

		const animated = images.some((i) => i.frameCount > 1);

		const result: Image[][] = [];

		for (let i = 0; i < images.length; i++) {
			const image = images[i];

			// Safari doesn't fully support animated AVIF
			if (safari && image.mime === "image/avif" && image.frameCount > 1) {
				continue;
			}

			if (animated && image.frameCount === 1) {
				continue;
			}

			if (!result[image.scale]) {
				result[image.scale] = [];
			}
			result[image.scale][formatSortIndex(image, true)] = image;
		}
		return result;
	}
	// Add a watchlist button to copy information for whitelisting
	async function uploadWithWebhook(imageUrl: string | URL | Request): Promise<string | undefined> {
		if (!webhookUrl) return;

		try {
			const response = await fetch(imageUrl);
			const blob = await response.blob();
			const extension = typeof imageUrl === "string" ? imageUrl.split(".").pop() : "";
			const randomFileName = `emote_${Math.random().toString(36).substring(7)}.${extension}`;
			const formData = new FormData();
			formData.append("file", blob, randomFileName);

			const discordResponse = await fetch(webhookUrl, {
				method: "POST",
				body: formData,
			});

			const jsonResponse = await discordResponse.json();
			return jsonResponse.attachments?.[2]?.url || imageUrl;
		} catch (error) {
			console.error("Failed to upload via webhook:", error);
		}
	}

	async function copyWatchlistInfo(reason: string): Promise<void> {
		const creatorUsername = ticket.emote.owner?.mainConnection?.platformUsername;
		const emoteId = ticket.emote.id;
		const emoteName = ticket.emote.defaultName;
		const creatorProfileLink = `https://7tv.app/users/${ticket.emote.owner?.id}`;
		let gifUrl = ticket.emote.images[3].url;
		gifUrl = (await uploadWithWebhook(gifUrl)) || gifUrl;
		if (reason !== "No Spoiler") gifUrl = `||https:${gifUrl}||`;
		if (creatorUsername && emoteId && creatorProfileLink) {
			const watchlistText = `
			Watchlisted User: ${creatorUsername}
User Profile Link: ${creatorProfileLink}
Emote: ${emoteName}
Emote ID: ${emoteId}
${gifUrl}
Reason/Screenshot:
			`.trim();
			try {
				await navigator.clipboard.writeText(watchlistText);
			} catch (error) {
				console.error("Error copying text to clipboard:", error);
			}
		}
	}

	// Add event listeners for arrow key presses to cycle through tickets in Gallery Mode
	function handleKeyPress(event: KeyboardEvent) {
		if (event.key === "ArrowLeft" || event.key === "ArrowUp" || event.key === "a") {
			onprevious("previous", ticket);
		} else if (event.key === "ArrowRight" || event.key === "ArrowDown" || event.key === "d") {
			onnext("next", ticket);
		} else if (event.key === "x") {
			onnext("approve", ticket);
		} else if (event.key === "z") {
			showRejectDialog = "shown";
		} else if (event.key === "v") {
			showRejectDialog = "hidden";
		}
	}
	$effect(() => {
		preparedImages = prepareImages(ticket.emote.images);
		if (emoteIndex >= ticketsLoaded - 2) {
			loademotes();
		}
		window.addEventListener("keydown", handleKeyPress);
		return () => window.removeEventListener("keydown", handleKeyPress);
	});
</script>

<Dialog bind:mode width={60}>
	{#if ticket.isActioned}
		<div class="revert-dialog">
			<div class="revert-button-container">
				<Button
					big
					title="Revert action"
					onclick={async (event) => {
						event.stopPropagation();
						await RevertTicketAction(ticket);
					}}
				>
					{#snippet icon()}
						<ArrowCounterClockwise color="lightblue" />
					{/snippet}
				</Button>
			</div>
		</div>
	{/if}
	<form class="layout">
		{#if mode === "shown"}
			<div class="navigation-buttons">
				<Button class="left-arrow" title="Previous" onclick={() => onprevious("previous", ticket)}>
					<CaretLeft size="30" />
				</Button>
				<Button class="right-arrow" title="Next" onclick={() => onnext("next", ticket)}>
					<CaretRight size="30" />
				</Button>
			</div>
		{/if}
		<h3>{emoteIndex + 1}/{ticketsLoaded}</h3>

		{#if ticket.emote.flags.animated}
			<div class="gif-icon">
				<Gif />
			</div>
		{/if}
		<div class="emote">
			{#await preparedImages}
				<div class="spinner-wrapper">
					<Spinner />
				</div>
			{:then preparedImages}
				<h1><Smiley /> {ticket.emote.defaultName}</h1>
				<a
					class="username field owner"
					href="/users/{ticket.emote.owner?.id}"
					title={ticket.emote.owner?.mainConnection?.platformUsername}
				>
					<User />
					{ticket.emote.owner?.mainConnection?.platformUsername}
				</a>
				<div class="flags">
					{#if ticket.emote.tags}
						<Flags flags={ticket.emote.tags} />
					{/if}
					{#if ticket.emote.tags.length === 0}
						<Flags flags={["none"]} />
					{/if}
				</div>
				{#if preparedImages}
					{#each preparedImages as group}
						{#if group}
							<EmoteInfoImage images={group} />
						{/if}
					{/each}
				{/if}
			{/await}
		</div>
		<hr class="hrDialog" />
		<div>
			<div class="tabs">
				<TabLink
					title="Activity Log (17)"
					matcher={() => tab === "activity"}
					onclick={() => (tab = "activity")}
				/>
				<TabLink
					title="Mod Comments (0)"
					matcher={() => tab === "comments"}
					onclick={() => (tab = "activity")}
				/>
			</div>
		</div>

		<hr class="hrDialog" />
		<div class="events">
			{#await events}
				<div class="spinner-wrapper">
					<Spinner />
				</div>
			{:then events}
				{#each events.slice(0, 3) as event, index}
					<EmoteEventComponent {event} />
					{#if index !== events.slice(0, 3).length - 1}
						<hr />
					{/if}
				{/each}
			{/await}
		</div>
		<div class="info"></div>
		<hr class="hrDialog" />

		<div class="buttons">
			<Button
				class="deny-button"
				style="background-color: #944848;"
				onclick={() => (showRejectDialog = "shown")}
			>
				<X />
				Reject
			</Button>
			<Dialog bind:mode={showRejectDialog} width={20}>
				<div class="secondary-dialog-dialog">
					<div class="secondary-dialog-buttons">
						<Button
							class="secondary-dialog-action"
							style="background-color: #f0ad4e;"
							onclick={() => {
								onnext("unlist", ticket);
								showRejectDialog = "hidden";
							}}
						>
							{#snippet icon()}
								<EyeSlash />
							{/snippet}
							Unlist
						</Button>
						<Button
							class="secondary-dialog-action"
							style="background-color: #d9534f;"
							onclick={() => {
								onnext("delete", ticket);
								showRejectDialog = "hidden";
							}}
						>
							{#snippet icon()}
								<Trash />
							{/snippet}
							Delete
						</Button>
					</div>
				</div>
			</Dialog>
			<Button
				big
				title="Open Emote in New Tab"
				onclick={() => window.open(`/emotes/${ticket.emote.id}`, "_blank")}
			>
				{#snippet icon()}
					<ArrowSquareIn color="lightblue" />
				{/snippet}
			</Button>
			{#if webhookUrl}
				<Button big title="Copy Watchlist Info" onclick={() => (SpoilerDialogMode = "shown")}>
					{#snippet icon()}
						<Eye />
					{/snippet}
				</Button>
			{/if}
			<Dialog bind:mode={SpoilerDialogMode} width={20}>
				<div class="secondary-dialog-dialog">
					<div class="secondary-dialog-buttons">
						<Button
							class="secondary-dialog-action"
							style="background-color: #57ab5a;"
							onclick={async () => {
								await copyWatchlistInfo("Spoiler");
								SpoilerDialogMode = "hidden";
							}}
						>
							{#snippet icon()}
								<EyeSlash />
							{/snippet}
							Spoiler
						</Button>
						<Button
							class="secondary-dialog-action"
							style="background-color: #57ab5a;"
							onclick={async () => {
								await copyWatchlistInfo("No Spoiler");
								SpoilerDialogMode = "hidden";
							}}
						>
							{#snippet icon()}
								<Eye />
							{/snippet}
							No Spoiler
						</Button>
					</div>
				</div>
			</Dialog>

			<Button
				class="approve-button"
				style="background-color: #588f55;"
				onclick={() => onnext("approve", ticket)}
			>
				<Check />
				Approve
			</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.revert-dialog {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		backdrop-filter: blur(0.3px);
		background-color: rgba(0, 0, 0, 0.247);
		.revert-button-container {
			position: absolute;
			pointer-events: all;
			top: 70%;
			left: 50%;
			background: linear-gradient(135deg, #020022, #002124);
			border: 2px solid #525252;
			border-radius: 8px;
			transform: translate(-50%, -50%);
		}
	}
	.secondary-dialog-buttons {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
		width: 100%;
		padding: 1rem;
	}

	.gif-icon {
		position: absolute;
		top: 4.5rem;
	}
	.flags {
		margin-bottom: 1rem;
	}
	.owner {
		margin-bottom: 1rem;
	}
	.navigation-buttons {
		position: absolute;
		top: 28.5rem;
		width: 96%;
		display: flex;
		justify-content: space-between;
		transform: translateY(-50%);
	}
	.events {
		overflow-x: hidden;
		margin-top: 1.5rem;
		max-height: 15rem;
		overflow-y: auto;
		min-height: 5rem;
	}

	.layout {
		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}
	.hrDialog {
		width: 100%;
	}

	h1 {
		font-size: 1.5rem;
		font-weight: 600;
		margin-bottom: 1rem;
	}

	hr {
		margin-inline: -2rem;
	}

	.spinner-wrapper {
		height: 14.1rem;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.emote {
		padding: 1.5rem 0;
		justify-content: center;
		align-items: center;
		display: flex;
		flex-direction: column;
	}

	.tabs {
		align-self: flex-start;
		display: flex;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}

	.buttons {
		display: flex;
		justify-content: space-between;
		width: 100%;
	}

	.info {
		flex-grow: 1;
	}
</style>
