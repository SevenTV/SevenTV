<script lang="ts">
	import type { Image } from "$/gql/graphql";
	import { writable } from "svelte/store";
	import type { DialogMode } from "../dialogs/dialog.svelte";
	import Dialog from "../dialogs/dialog.svelte";
	import { isSafari } from "$/lib/utils";
	import { formatSortIndex } from "../responsive-image.svelte";
	import EmoteInfoImage from "../emotes/emote-info-image.svelte";
	import { deleteEmote } from "$/lib/emoteMutations";
	import { Smiley, User } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import type { ModRequestsTicket } from "./emote-tickets.svelte";
	let inputValue = writable("");

	interface Props {
		mode: DialogMode;
		ticket: ModRequestsTicket;
	}

	let { mode = $bindable("hidden"), ticket }: Props = $props();
	const emote = ticket.emote;

	async function deleteEmoteCall() {
		const output = await deleteEmote(ticket.emote.id);
		if (output) {
			ticket.isActioned = true;
		} else {
			console.error("Failed to delete emote");
		}
		inputValue.set("");
		mode = "hidden";
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
</script>

<Dialog bind:mode width={42}>
	<div class="modal">
		<h1>Delete {emote.defaultName}</h1>
		<h2>
			{$t("dialogs.deleteEmote.warning_message")}
		</h2>

		<div class="emote">
			{#each prepareImages(emote.images) as group}
				{#if group}
					<EmoteInfoImage images={group} />
				{/if}
			{/each}
			<div class="info">
				<div>
					<a class="emote-name field" href="/emotes/{emote.id}" title={emote.defaultName}>
						<Smiley />
						{emote.defaultName}
					</a>
				</div>
				<div>
					<a
						class="username field owner"
						href="/users/{emote.owner?.id}"
						title={emote.owner?.mainConnection?.platformUsername}
					>
						<User />
						{emote.owner?.mainConnection?.platformUsername}
					</a>
				</div>
			</div>
		</div>
		<button class="deleteButton" onclick={deleteEmoteCall}>Delete {emote.defaultName}</button>
	</div>
</Dialog>

<style lang="scss">
	.info {
		flex-grow: 1;
		margin-block: auto;

		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.75rem;
	}
	.modal {
		padding: 20px;
		border-radius: 10px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		align-items: center;
		text-align: center;
	}
	.deleteButton {
		background-color: #d61a39;
		color: white;
		border: none;
		border-radius: 5px;
		padding: 10px;
		cursor: pointer;
		transition: 0.3s;
	}
</style>
