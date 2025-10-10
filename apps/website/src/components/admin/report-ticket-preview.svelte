<script lang="ts">
	import { Check, Smiley, User, Spinner, ArrowCounterClockwise } from "phosphor-svelte";
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
	import { t } from "svelte-i18n";
	import TabLink from "../tab-link.svelte";
	import type { ModRequestsTicket } from "./report-tickets.svelte";
	interface Props {
		mode: DialogMode;
		ticket: ModRequestsTicket;
	}

	let { mode = $bindable("hidden"), ticket = $bindable() }: Props = $props();
	let events = $derived(loadEvents(ticket.emote.id));
	let preparedImages = $state<Image[][]>([]);
	let tab: "activity" | "comments" = $state("activity");

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

	$effect(() => {
		preparedImages = prepareImages(ticket.emote.images);
	});
</script>

<Dialog bind:mode width={60}>
	{#if ticket.isActioned}
		<div class="revert-dialog">
			<div class="revert-button-container">
				<Button big title="Revert action">
					{#snippet icon()}
						<ArrowCounterClockwise color="lightblue" />
					{/snippet}
				</Button>
			</div>
		</div>
	{/if}
	<form class="layout">
		<div class="header-container">
			<div class="header-item">
				<h2 class="header">{$t("pages.admin.tickets.reports_table.reported_by")}:</h2>
				<div class="info-user">
					<img
						class="avatar"
						src={ticket.message.actor.avatar_url}
						alt={ticket.message.actor.display_name}
					/>
					<p>{ticket.message.actor.display_name}</p>
				</div>
			</div>

			<div class="header-item">
				<h2 class="header">{$t("pages.admin.tickets.reports_table.subject")}:</h2>
				<p class="info">{ticket.message.subject}</p>
			</div>

			<div class="header-item">
				<h2 class="header">{$t("pages.admin.tickets.reports_table.details")}:</h2>
				<p class="info">{ticket.message.body}</p>
			</div>
		</div>
		<hr class="hrDialog" />
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

		<div class="action-buttons">
			<Button
				class="close-button"
				style="background-color: #588f55;"
				onclick={() => console.log("Mark as closed")}
				big
			>
				<Check />
				{$t("pages.admin.tickets.reports_actions.close")}
			</Button>
			<Button
				class="assign-button"
				style="background-color: #f0ad4e;"
				onclick={() => console.log("Assign self")}
				big
			>
				<User />
				{$t("pages.admin.tickets.reports_actions.assign_self")}
			</Button>
			<Button
				class="comment-button"
				style="background-color: #944848;"
				onclick={() => console.log("Add comment")}
				big
			>
				<Smiley />
				{$t("pages.admin.tickets.reports_actions.add_comment")}
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
		backdrop-filter: blur(1px);
		background-color: rgba(0, 0, 0, 0.5);
		.revert-button-container {
			position: absolute;
			top: 70%;
			left: 50%;
			background: linear-gradient(135deg, #020022, #002124);
			border: 2px solid #525252;
			border-radius: 8px;
			transform: translate(-50%, -50%);
		}
	}

	.flags {
		margin-bottom: 1rem;
	}
	.owner {
		margin-bottom: 1rem;
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

	.header-container {
		display: flex;
		justify-content: space-between;
		width: 100%;
		margin-bottom: 1.5rem;
		gap: 1rem;
	}

	.header-item {
		flex: 1;
		padding: 1rem;
		border-radius: 0.5rem;
	}

	.header {
		font-size: 1.25rem;
		font-weight: bold;
		margin-bottom: 0.5rem;
		color: var(--text-primary);
	}

	.info {
		font-size: 1rem;
		color: var(--text-secondary);
		word-wrap: break-word;
	}

	.info-user {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 8px;
	}

	.avatar {
		width: 2rem;
		height: 2rem;
		border-radius: 50%;
	}

	.action-buttons {
		position: relative;
		bottom: 0rem;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		flex-direction: row;
		gap: 1.5rem;
		width: 100%;
		justify-content: center;
		z-index: 1000;
	}
</style>
