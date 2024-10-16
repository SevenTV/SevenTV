<script lang="ts">
	import {
		ArrowBendDownRight,
		Plus,
		FolderPlus,
		NotePencil,
		CaretDown,
		PaperPlaneRight,
		ArrowsMerge,
		Download,
		Trash,
		Flag,
	} from "phosphor-svelte";
	import Tags from "$/components/emotes/tags.svelte";
	import Flags, { emoteToFlags } from "$/components/flags.svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";
	import DropDown from "../drop-down.svelte";
	import { DialogMode } from "../dialogs/dialog.svelte";
	import AddEmoteDialog from "../dialogs/add-emote-dialog.svelte";
	import EditEmoteDialog from "../dialogs/edit-emote-dialog.svelte";
	import TransferEmoteDialog from "../dialogs/transfer-emote-dialog.svelte";
	import ReportEmoteDialog from "../dialogs/report-emote-dialog.svelte";
	import DeleteEmoteDialog from "../dialogs/delete-emote-dialog.svelte";
	import MenuButton from "../input/menu-button.svelte";
	import type { Emote, Image } from "$/gql/graphql";
	import EmoteInfoImage from "./emote-info-image.svelte";
	import { formatSortIndex } from "../responsive-image.svelte";
	import UserProfilePicture from "../user-profile-picture.svelte";
	import EmoteLoadingPlaceholder from "../emote-loading-placeholder.svelte";

	export let data: Emote | null;

	enum MoreMenuMode {
		Root,
		DownloadFormat,
		DownloadSize,
	}

	let moreMenuMode = MoreMenuMode.Root;
	let downloadFormat: string;

	function clickFormat(format: string) {
		downloadFormat = format;
		moreMenuMode = MoreMenuMode.DownloadSize;
	}

	function download(size: number) {
		if (!downloadFormat) return;
		alert(`downloading ${downloadFormat} at ${size}x`);
	}

	let addEmoteDialogMode = DialogMode.Hidden;
	let editDialogMode = DialogMode.Hidden;
	let transferDialogMode = DialogMode.Hidden;
	let deleteDialogMode = DialogMode.Hidden;
	let reportDialogMode = DialogMode.Hidden;

	function prepareImages(images: Image[]): Image[][] {
		const animated = images.some((i) => i.frameCount > 1);

		const result: Image[][] = [];

		for (let i = 0; i < images.length; i++) {
			const image = images[i];

			if (animated && image.frameCount === 1) {
				continue;
			}

			if (!result[image.scale]) {
				result[image.scale] = [];
			}

			result[image.scale][formatSortIndex(image)] = image;
		}

		return result;
	}
</script>

{#if !$$slots.default}
	<AddEmoteDialog bind:mode={addEmoteDialogMode} />
	<EditEmoteDialog bind:mode={editDialogMode} />
	<TransferEmoteDialog bind:mode={transferDialogMode} />
	<ReportEmoteDialog bind:mode={reportDialogMode} />
	<DeleteEmoteDialog bind:mode={deleteDialogMode} />
{/if}
{#if data}
	<div class="top-bar">
		<Flags flags={emoteToFlags(data)} />
		{#if data.owner}
			<a href="/users/{data.owner.id}" class="user-info">
				<UserProfilePicture user={data.owner} />
				<div class="name-container">
					<span class="username" style:color={data.owner.highestRoleColor?.hex}>{data.owner.mainConnection?.platformDisplayName}</span>
					{#if data.attribution.length > 0}
						<div class="artists">
							<ArrowBendDownRight size="0.75rem" color="var(--text-light)" />
							{#each data.attribution as artist}
								{#if artist.user}
									<a href="/users/{artist.user.id}" class="profile">
										<UserProfilePicture user={artist.user} size={16} />
									</a>
								{/if}
							{/each}
						</div>
					{/if}
				</div>
			</a>
		{/if}
	</div>
{/if}
<div class="emote-info">
	<div class="heading">
		{#if data}
			<h1>{data.defaultName}</h1>
			<Tags tags={data.tags} />
		{:else}
			<h1>Loading</h1>
		{/if}
	</div>
	<div class="previews">
		{#if data}
			{#each prepareImages(data.images) as group}
				{#if group}
					<EmoteInfoImage images={group} />
				{/if}
			{/each}
		{:else}
			<EmoteLoadingPlaceholder index={0} size={32} />
			<EmoteLoadingPlaceholder index={2} size={64} />
			<EmoteLoadingPlaceholder index={3} size={96} />
			<EmoteLoadingPlaceholder index={3} size={128} />
		{/if}
	</div>
	{#if data}
		<div class="buttons">
			<slot>
				<Button primary>
					<Plus slot="icon" />
					{$t("pages.emote.use_emote")}
				</Button>
				<Button secondary on:click={() => (addEmoteDialogMode = DialogMode.Shown)}>
					<FolderPlus slot="icon" />
					{$t("pages.emote.add_to")}
				</Button>
				<Button secondary hideOnMobile on:click={() => (editDialogMode = DialogMode.Shown)}>
					<NotePencil slot="icon" />
					{$t("labels.edit")}
				</Button>
				<Button secondary hideOnDesktop on:click={() => (editDialogMode = DialogMode.Shown)}>
					<NotePencil slot="icon" />
				</Button>
				<DropDown>
					<Button secondary hideOnMobile on:click={() => (moreMenuMode = MoreMenuMode.Root)}>
						{$t("labels.more")}
						<CaretDown slot="icon-right" />
					</Button>
					<Button secondary hideOnDesktop>
						<CaretDown slot="icon" />
					</Button>
					<div slot="dropdown" class="dropdown">
						{#if moreMenuMode === MoreMenuMode.Root}
							<MenuButton on:click={() => (transferDialogMode = DialogMode.Shown)}>
								<PaperPlaneRight />
								{$t("pages.emote.transfer")}
							</MenuButton>
							<MenuButton>
								<ArrowsMerge style="transform: rotate(-90deg)" />
								{$t("pages.emote.merge")}
							</MenuButton>
							<MenuButton showCaret on:click={() => (moreMenuMode = MoreMenuMode.DownloadFormat)}>
								<Download />
								{$t("labels.download")}
							</MenuButton>
							<MenuButton on:click={() => (reportDialogMode = DialogMode.Shown)}>
								<Flag />
								{$t("labels.report")}
							</MenuButton>
							<hr />
							<MenuButton
								style="color: var(--danger)"
								on:click={() => (deleteDialogMode = DialogMode.Shown)}
							>
								<Trash />
								{$t("labels.delete")}
							</MenuButton>
						{:else if moreMenuMode === MoreMenuMode.DownloadFormat}
							{#each ["GIF", "WEBP", "AVIF"] as format}
								<MenuButton showCaret on:click={() => clickFormat(format)}>
									{format}
								</MenuButton>
							{/each}
						{:else if moreMenuMode === MoreMenuMode.DownloadSize}
							{#each [1, 2, 3, 4] as size}
								<MenuButton on:click={() => download(size)}>
									{size}x {$t("pages.emote.size")}
								</MenuButton>
							{/each}
						{/if}
					</div>
				</DropDown>
			</slot>
		</div>
	{/if}
</div>

<style lang="scss">
	.top-bar {
		display: flex;
		justify-content: space-between;
		flex-direction: row-reverse;
		align-items: center;
		gap: 1rem;
	}

	.user-info {
		display: flex;
		align-items: center;
		column-gap: 0.5rem;
		text-decoration: none;

		.name-container {
			display: flex;
			flex-direction: column;
			gap: 0.25rem;
		}

		.username {
			font-weight: 500;
			color: var(--text);
		}

		.artists {
			display: flex;
			gap: 0.25rem;
		}
	}

	.emote-info {
		margin-top: 0.5rem;

		display: flex;
		flex-direction: column;
		justify-content: space-between;
		align-items: center;
		gap: 2rem;

		.heading {
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: 1rem;

			h1 {
				font-size: 1.25rem;
				font-weight: 600;
			}
		}

		.previews {
			display: flex;
			align-items: flex-end;
			gap: 2.25rem;
		}

		.buttons {
			display: flex;
			gap: 0.5rem;
		}

		.dropdown {
			display: flex;
			flex-direction: column;
		}
	}

	@media screen and (max-width: 960px) {
		.emote-info .previews {
			gap: 0.75rem;
		}
	}
</style>
