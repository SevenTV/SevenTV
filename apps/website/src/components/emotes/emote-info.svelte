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
	import type { DialogMode } from "../dialogs/dialog.svelte";
	import type { Snippet } from "svelte";
	import UserName from "../user-name.svelte";
	import { user } from "$/lib/auth";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { addEmoteToSet } from "$/lib/setMutations";
	import Spinner from "../spinner.svelte";

	type MoreMenuMode = "root" | "download-format" | "download-size";

	let { data, children }: { data: Emote | null; children?: Snippet } = $props();

	let moreMenuMode: MoreMenuMode = $state("root");
	let downloadFormat: string | undefined = $state();

	function clickFormat(format: string) {
		downloadFormat = format;
		moreMenuMode = "download-size";
	}

	function download(size: number) {
		if (!downloadFormat) return;
		alert(`downloading ${downloadFormat} at ${size}x`);
	}

	let addEmoteDialogMode: DialogMode = $state("hidden");
	let editDialogMode: DialogMode = $state("hidden");
	let transferDialogMode: DialogMode = $state("hidden");
	let deleteDialogMode: DialogMode = $state("hidden");
	let reportDialogMode: DialogMode = $state("hidden");

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

	let useEmoteLoading = $state(false);

	async function useEmote() {
		if (!$defaultEmoteSet || !data) {
			return;
		}

		useEmoteLoading = true;
		await addEmoteToSet($defaultEmoteSet, data.id);
		useEmoteLoading = false;
	}
</script>

{#if !children && data}
	<!-- Rerender when opened -->
	{#if addEmoteDialogMode !== "hidden"}
		<AddEmoteDialog bind:mode={addEmoteDialogMode} {data} />
	{/if}
	<EditEmoteDialog bind:mode={editDialogMode} />
	<TransferEmoteDialog bind:mode={transferDialogMode} {data} />
	<ReportEmoteDialog bind:mode={reportDialogMode} />
	<DeleteEmoteDialog bind:mode={deleteDialogMode} {data} />
{/if}
{#if data}
	<div class="top-bar">
		<Flags flags={emoteToFlags(data)} />
		{#if data.owner}
			<div class="user-info">
				<a href="/users/{data.owner.id}">
					<UserProfilePicture user={data.owner} />
				</a>
				<div class="name-container">
					<a href="/users/{data.owner.id}" class="username">
						<UserName user={data.owner} />
					</a>
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
			</div>
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
			{#snippet fallbackChildren()}
				{#if $user}
					{#if $defaultEmoteSet}
						<Button primary onclick={useEmote} disabled={useEmoteLoading}>
							{#snippet icon()}
								{#if useEmoteLoading}
									<Spinner />
								{:else}
									<Plus />
								{/if}
							{/snippet}
							{$t("pages.emote.use_emote")}
						</Button>
					{/if}
					<Button
						primary={!$defaultEmoteSet}
						secondary={!!$defaultEmoteSet}
						onclick={() => (addEmoteDialogMode = "shown")}
					>
						{#snippet icon()}
							<FolderPlus />
						{/snippet}
						{$t("pages.emote.add_to")}
					</Button>
					<Button secondary hideOnMobile onclick={() => (editDialogMode = "shown")}>
						{#snippet icon()}
							<NotePencil />
						{/snippet}
						{$t("labels.edit")}
					</Button>
					<Button secondary hideOnDesktop onclick={() => (editDialogMode = "shown")}>
						{#snippet icon()}
							<NotePencil />
						{/snippet}
					</Button>
				{/if}
				<DropDown>
					<Button secondary hideOnMobile onclick={() => (moreMenuMode = "root")}>
						{$t("labels.more")}
						{#snippet iconRight()}
							<CaretDown />
						{/snippet}
					</Button>
					<Button secondary hideOnDesktop>
						{#snippet icon()}
							<CaretDown />
						{/snippet}
					</Button>
					{#snippet dropdown()}
						<div class="dropdown">
							{#if moreMenuMode === "root"}
								<MenuButton onclick={() => (transferDialogMode = "shown")}>
									<PaperPlaneRight />
									{$t("pages.emote.transfer")}
								</MenuButton>
								<MenuButton>
									<ArrowsMerge style="transform: rotate(-90deg)" />
									{$t("pages.emote.merge")}
								</MenuButton>
								<MenuButton showCaret onclick={() => (moreMenuMode = "download-format")}>
									<Download />
									{$t("labels.download")}
								</MenuButton>
								<MenuButton onclick={() => (reportDialogMode = "shown")}>
									<Flag />
									{$t("labels.report")}
								</MenuButton>
								<hr />
								<MenuButton
									style="color: var(--danger)"
									onclick={() => (deleteDialogMode = "shown")}
								>
									<Trash />
									{$t("labels.delete")}
								</MenuButton>
							{:else if moreMenuMode === "download-format"}
								{#each ["GIF", "WEBP", "AVIF"] as format}
									<MenuButton showCaret onclick={() => clickFormat(format)}>
										{format}
									</MenuButton>
								{/each}
							{:else if moreMenuMode === "download-size"}
								{#each [1, 2, 3, 4] as size}
									<MenuButton onclick={() => download(size)}>
										{size}x {$t("pages.emote.size")}
									</MenuButton>
								{/each}
							{/if}
						</div>
					{/snippet}
				</DropDown>
			{/snippet}

			{@render (children ?? fallbackChildren)()}
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
			text-decoration: none;
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
