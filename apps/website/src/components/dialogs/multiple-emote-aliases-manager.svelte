<script lang="ts">
	import type { Emote, EmoteSetEmote } from "$/gql/graphql";
	import { addEmoteToSet, removeEmoteFromSet, renameEmoteInSet } from "$/lib/setMutations";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import Spinner from "../spinner.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";
	import PencilSimple from "phosphor-svelte/lib/PencilSimple";
	import Plus from "phosphor-svelte/lib/Plus";
	import Trash from "phosphor-svelte/lib/Trash";

	interface Props {
		mode: DialogMode;
		data: Emote;
		multipleEmoteAliasesManagerData?: {
			all: EmoteSetEmote[];
			count: number;
			status: boolean;
			emote: Emote;
		};
		emoteSetName?: string;
		emoteSetId?: string;
	}

	let {
		mode = $bindable("hidden"),
		data = $bindable(),
		multipleEmoteAliasesManagerData = $bindable(),
		emoteSetId = $bindable(),
		emoteSetName = $bindable(),
	}: Props = $props();

	let visibleCount = $state(20);
	let loadingIndex = $state<number | null>(null);
	let deletingIndex = $state<number | null>(null);
	let isAdding = $state(false);
	let newAliasName = $state("");
	let totalEmotesWithAliases = $derived(multipleEmoteAliasesManagerData?.all.length ?? 0);

	let savedAliases = $state<string[]>([]);

	$effect(() => {
		if (multipleEmoteAliasesManagerData?.all) {
			if (savedAliases.length !== multipleEmoteAliasesManagerData.all.length) {
				savedAliases = multipleEmoteAliasesManagerData.all.map((e) => e.alias);
			}
		}
	});

	let visibleEmotes = $derived(multipleEmoteAliasesManagerData?.all.slice(0, visibleCount) ?? []);

	// used try catch on all the functions below for future mutation/query
	// functions remake throwing errors
	async function handleAdd() {
		if (!emoteSetId || !newAliasName.trim() || !multipleEmoteAliasesManagerData) return;

		isAdding = true;
		const aliasToAdd = newAliasName.trim();
		try {
			await addEmoteToSet(emoteSetId, data.id, aliasToAdd);
			const newEntry = {
				...multipleEmoteAliasesManagerData.all[0],
				alias: aliasToAdd,
			};
			multipleEmoteAliasesManagerData.all = [newEntry, ...multipleEmoteAliasesManagerData.all];
			multipleEmoteAliasesManagerData.count = multipleEmoteAliasesManagerData.all.length;
			savedAliases = [aliasToAdd, ...savedAliases];
			newAliasName = "";
		} catch (error) {
			console.error("Failed to add alias:", error);
		} finally {
			isAdding = false;
		}
	}

	async function handleRename(emoteInSet: EmoteSetEmote, index: number) {
		if (!emoteSetId || !multipleEmoteAliasesManagerData) return;
		loadingIndex = index;
		try {
			await renameEmoteInSet(emoteSetId, data.id, emoteInSet.alias, savedAliases[index]);
			savedAliases[index] = emoteInSet.alias;
			multipleEmoteAliasesManagerData.all = [...multipleEmoteAliasesManagerData.all];
		} catch (error) {
			console.error("Failed to rename:", error);
		} finally {
			loadingIndex = null;
		}
	}
	async function handleDelete(index: number) {
		if (!emoteSetId || !multipleEmoteAliasesManagerData) return;
		deletingIndex = index;
		const aliasToRemove = multipleEmoteAliasesManagerData.all[index].alias;

		try {
			await removeEmoteFromSet(emoteSetId, data.id, aliasToRemove);
			multipleEmoteAliasesManagerData.all = multipleEmoteAliasesManagerData.all.filter(
				(_, i) => i !== index,
			);
			multipleEmoteAliasesManagerData.count = multipleEmoteAliasesManagerData.all.length;
			savedAliases = savedAliases.filter((_, i) => i !== index);
		} catch (error) {
			console.error("Failed to delete:", error);
		} finally {
			deletingIndex = null;
		}
	}

	function handleScroll(e: Event) {
		const target = e.target as HTMLElement;
		const bottom = target.scrollHeight - target.scrollTop <= target.clientHeight + 100;
		if (
			bottom &&
			multipleEmoteAliasesManagerData &&
			visibleCount < multipleEmoteAliasesManagerData.all.length
		) {
			visibleCount += 20;
		}
	}
</script>

<EmoteDialog
	width={60}
	title={$t("dialogs.manage_aliases.title", { values: { emote: data.defaultName ?? "" } })}
	bind:mode
	{data}
>
	<div class="add-section">
		<div class="alias-row add-row">
			<div class="preview-column"><EmotePreview {data} emoteOnly /></div>
			<div class="input-column">
				<TextInput
					placeholder={$t("dialogs.manage_aliases.add_placeholder", {
						default: $t("dialogs.manage_aliases.add_placeholder", { default: "NewAlias" }),
					})}
					bind:value={newAliasName}
				/>
			</div>
			<div class="actions-column">
				<Button disabled={!newAliasName.trim() || isAdding} onclick={handleAdd}>
					{#if isAdding}
						<Spinner size={16} />
					{:else}
						<Plus weight="bold" size={18} />
					{/if}
				</Button>
			</div>
		</div>
	</div>

	<div class="divider"></div>

	<h1>{totalEmotesWithAliases} Emotes</h1>

	<div class="alias-list" onscroll={handleScroll}>
		{#if multipleEmoteAliasesManagerData?.all}
			{#each visibleEmotes as emoteInSet, i (emoteInSet.alias + i)}
				<div class="alias-row" class:modified={emoteInSet.alias !== savedAliases[i]}>
					<div class="preview-column"><EmotePreview {data} emoteOnly /></div>
					<div class="input-column">
						<TextInput placeholder={data.defaultName} bind:value={emoteInSet.alias} />
					</div>
					<div class="actions-column">
						<Button onclick={() => handleRename(emoteInSet, i)} disabled={loadingIndex !== null}>
							{#if loadingIndex === i}
								<Spinner size={16} />
							{:else}
								<PencilSimple weight={"fill"} size={18} />
							{/if}
						</Button>
						<Button secondary onclick={() => handleDelete(i)} disabled={deletingIndex !== null}>
							{#if deletingIndex === i}
								<Spinner size={16} />
							{:else}
								<Trash weight="bold" size={18} />
							{/if}
						</Button>
					</div>
				</div>
			{/each}
		{/if}
	</div>

	{#snippet buttons()}
		<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.close")}</Button>
	{/snippet}
</EmoteDialog>

<style lang="scss">
	.add-section {
		padding-bottom: 1rem;
	}
	.add-row {
		background: var(--background-higher) !important;
		border-color: var(--primary-main) !important;
	}
	.divider {
		height: 1px;
		background: var(--border-color);
		margin-bottom: 1rem;
		opacity: 0.5;
	}

	.alias-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-height: 400px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.alias-row {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.6rem;
		background: var(--background-float);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		transition: all 0.2s ease;
		&.modified {
			border-color: var(--primary-main);
			background: var(--background-higher);
			box-shadow: 0 0 0 1px var(--primary-main);
		}
		.preview-column {
			width: 32px;
			height: 32px;

			flex-shrink: 0;
		}
		.input-column {
			flex-grow: 1;
		}
		.actions-column {
			display: flex;

			gap: 0.4rem;
		}
	}

	.loading-trigger {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 1rem;
		color: var(--text-muted);
		font-size: 0.8rem;
	}
</style>
