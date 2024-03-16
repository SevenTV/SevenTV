<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import ImagePreview from "$/components/image-preview.svelte";
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
	import type { LayoutData } from "./$types";
	import Tags from "$/components/emotes/tags.svelte";
	import Flags from "$/components/flags.svelte";
	import EditEmoteDialog from "$/components/dialogs/edit-emote-dialog.svelte";
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import AddEmoteDialog from "$/components/dialogs/add-emote-dialog.svelte";
	import DropDown from "$/components/drop-down.svelte";
	import TransferEmoteDialog from "$/components/dialogs/transfer-emote-dialog.svelte";
	import DeleteEmoteDialog from "$/components/dialogs/delete-emote-dialog.svelte";
	import ReportEmoteDialog from "$/components/dialogs/report-emote-dialog.svelte";
	import MenuButton from "$/components/input/menu-button.svelte";
	import { t } from "svelte-i18n";

	export let data: LayoutData;

	let addEmoteDialogMode = DialogMode.Hidden;
	let editDialogMode = DialogMode.Hidden;
	let transferDialogMode = DialogMode.Hidden;
	let deleteDialogMode = DialogMode.Hidden;
	let reportDialogMode = DialogMode.Hidden;

	enum MoreMenuMode {
		Root,
		DownloadFormat,
		DownloadSize,
	};

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
</script>

<svelte:head>
	<title>{data.id} - 7TV</title>
</svelte:head>

<AddEmoteDialog bind:mode={addEmoteDialogMode} />
<EditEmoteDialog bind:mode={editDialogMode} />
<TransferEmoteDialog bind:mode={transferDialogMode} />
<ReportEmoteDialog bind:mode={reportDialogMode} />
<DeleteEmoteDialog bind:mode={deleteDialogMode} />
<div class="layout">
	<div>
		<div class="top-bar">
			<a href="/user/ayyybubu" class="user-info">
				<img
					src="/test-profile-pic.jpeg"
					width="44"
					height="44"
					alt="profile"
					class="profile-picture"
				/>
				<span class="username">ayyybubu</span>
				<div class="artists">
					<ArrowBendDownRight size="0.75rem" color="var(--text-light)" />
					<a href="/user/ayyybubu" class="profile">
						<img
							src="/test-profile-pic.jpeg"
							width="16"
							height="16"
							alt="ayyybubu"
							title="ayyybubu"
							class="artist-picture"
						/>
					</a>
				</div>
			</a>
			<Flags flags={["active", "global", "trending", "overlaying"]} />
		</div>

		<div class="emote-info">
			<div class="heading">
				<h1>{data.id}</h1>
				<Tags tags={["tag1", "tag2", "tag3"]} />
			</div>
			<div class="previews">
				<ImagePreview size={32} />
				<ImagePreview size={64} />
				<ImagePreview size={96} />
				<ImagePreview size={128} />
			</div>
			<div class="buttons">
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
					<Button secondary hideOnMobile>
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
			</div>
		</div>
	</div>
	<div class="tabs">
		<slot />
	</div>
</div>

<style lang="scss">
	.layout {
		width: 100%;
		max-width: 80rem;
		margin-inline: auto;

		padding: 1.25rem;
		min-height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		& > * {
			background-color: var(--bg-medium);
			border: 1px solid var(--layout-border);
			border-radius: 0.5rem;
			padding: 1rem;
		}
	}

	.top-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
	}

	.user-info {
		display: grid;
		grid-template-columns: auto auto;
		grid-template-rows: auto auto;
		align-items: center;
		column-gap: 0.5rem;
		row-gap: 0.25rem;
		text-decoration: none;

		.profile-picture {
			grid-row: 1 / -1;

			width: 2.75rem;
			height: 2.75rem;
			border-radius: 50%;
			border: 2px solid var(--staff);
		}

		.username {
			color: var(--staff);
			font-weight: 500;
		}

		.artists {
			display: flex;
			gap: 0.25rem;

			.artist-picture {
				width: 1rem;
				height: 1rem;
				border-radius: 50%;
				border: 1px solid var(--text);
			}
		}
	}

	.emote-info {
		margin-top: 0.5rem;

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

		display: flex;
		flex-direction: column;
		justify-content: space-between;
		align-items: center;
		gap: 2rem;

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

	.tabs {
		flex-grow: 1;
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
		}

		.emote-info .previews {
			gap: 0.75rem;
		}
	}
</style>
