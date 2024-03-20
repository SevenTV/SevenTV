<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import EmoteContainer from "$/components/layout/emote-container.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
	import Tags from "$/components/emotes/tags.svelte";
	import {
		Copy,
		Lightning,
		LightningSlash,
		MagnifyingGlass,
		NotePencil,
		Trash,
	} from "phosphor-svelte";
	import type { PageData } from "./$types";
	import Select from "$/components/input/select.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import Toggle from "$/components/input/toggle.svelte";
	import Flags from "$/components/flags.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import EditEmoteSetDialog from "$/components/dialogs/edit-emote-set-dialog.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import CopyEmotesDialog from "$/components/dialogs/copy-emotes-dialog.svelte";
	import RemoveEmotesDialog from "$/components/dialogs/remove-emotes-dialog.svelte";
	import { t } from "svelte-i18n";

	export let data: PageData;

	let enabled = false;
	let selectionMode = false;
	let editDialogMode = DialogMode.Hidden;
	let copyEmotesDialogMode = DialogMode.Hidden;
	let removeEmotesDialogMode = DialogMode.Hidden;
</script>

<svelte:head>
	<title>{data.id} - {$t("page_titles.suffix")}</title>
</svelte:head>

<EditEmoteSetDialog bind:mode={editDialogMode} />
<CopyEmotesDialog bind:mode={copyEmotesDialogMode} />
<RemoveEmotesDialog bind:mode={removeEmotesDialogMode} />
<div class="layout">
	<div class="set-info">
		<h1>{data.id}</h1>
		<Flags flags={["verified", "public"]} style="position: absolute; top: 1rem; right: 1rem;" />
		<Tags tags={["lorem", "tag"]} />
		<div class="progress">
			<progress max="600" value="100" />
			100/600
		</div>
	</div>
	<div class="controls">
		<div class="buttons">
			<Button secondary on:click={() => (selectionMode = !selectionMode)} hideOnDesktop>
				{$t("labels.select")}
				<Toggle bind:value={selectionMode} slot="icon-right" />
			</Button>
			<HideOn mobile={selectionMode}>
				<Button primary on:click={() => (enabled = !enabled)}>
					{#if enabled}
						{$t("labels.disable")}
					{:else}
						{$t("labels.enable")}
					{/if}
					<svelte:fragment slot="icon-right">
						{#if enabled}
							<LightningSlash />
						{:else}
							<Lightning />
						{/if}
					</svelte:fragment>
				</Button>
			</HideOn>
			<Button secondary hideOnMobile on:click={() => (editDialogMode = DialogMode.Shown)}>
				{$t("labels.edit")}
				<NotePencil slot="icon-right" />
			</Button>
			<Button secondary hideOnMobile>
				{$t("pages.emote_set.copy_set")}
				<Copy slot="icon-right" />
			</Button>
			{#if !selectionMode}
				<Button secondary hideOnDesktop on:click={() => (editDialogMode = DialogMode.Shown)}>
					<NotePencil slot="icon-right" />
				</Button>
				<Button secondary hideOnDesktop>
					<Copy slot="icon-right" />
				</Button>
			{/if}
			<Button secondary on:click={() => (selectionMode = !selectionMode)} hideOnMobile>
				{$t("labels.selection_mode")}
				<Toggle bind:value={selectionMode} slot="icon-right" />
			</Button>
			{#if selectionMode}
				<Button on:click={() => (copyEmotesDialogMode = DialogMode.Shown)}>
					<Copy slot="icon" />
				</Button>
				<Button>
					<NotePencil slot="icon" />
				</Button>
				<Button on:click={() => (removeEmotesDialogMode = DialogMode.Shown)}>
					<Trash slot="icon" />
				</Button>
			{/if}
		</div>
		<div class="buttons">
			<Select
				options={[
					{ value: "none", label: $t("labels.no_filters") },
					{ value: "filters", label: $t("labels.filters") },
				]}
			/>
			<TextInput placeholder={$t("labels.search")}>
				<MagnifyingGlass slot="icon" />
			</TextInput>
			<LayoutButtons />
		</div>
	</div>
	<div class="content">
		<EmoteContainer layout={$emotesLayout}>
			{#each Array(100) as _, i}
				<EmotePreview
					name="emoteSetEmote{i}"
					index={i}
					emoteOnly={$emotesLayout === Layout.SmallGrid}
					{selectionMode}
				/>
			{/each}
		</EmoteContainer>
	</div>
</div>

<style lang="scss">
	.layout {
		padding: 1.25rem;
		padding-bottom: 0;
		height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	progress[value] {
		-webkit-appearance: none;
		-moz-appearance: none;
		appearance: none;
		border: none;

		width: 100%;
		height: 0.5rem;

		&,
		&::-webkit-progress-bar {
			border-radius: 0.25rem;
			background-color: var(--secondary);
		}

		&::-moz-progress-bar {
			border-radius: 0.25rem;
			background-color: var(--primary);
		}

		&::-webkit-progress-value {
			border-radius: 0.25rem;
			background-color: var(--primary);
		}
	}

	.set-info {
		position: relative;
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.75rem;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		h1 {
			text-align: center;
			font-size: 1.125rem;
			font-weight: 500;
		}

		.progress {
			display: flex;
			align-items: center;
			gap: 0.75rem;

			font-size: 0.875rem;
			font-weight: 500;

			progress {
				flex-grow: 1;
			}
		}
	}

	.controls {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap-reverse;
		justify-content: space-between;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.content {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;
		margin-right: -1.25rem;
		padding-right: 1.25rem;
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
			// Scroll whole layout on mobile
			height: auto;
		}

		.content {
			margin-right: 0;
			padding-right: 0;
		}
	}
</style>
