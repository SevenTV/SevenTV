<script lang="ts">
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import { type Emote, type EmoteSet } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import Spinner from "./spinner.svelte";
	import { browser } from "$app/environment";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import DropDown from "../components/drop-down.svelte";
	import { CaretDown, MagnifyingGlass, Minus, PencilSimple, Plus, Warning } from "phosphor-svelte";
	import TextInput from "./input/text-input.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";
	import Button from "./input/button.svelte";
	import { t } from "svelte-i18n";
	import MenuButton from "./input/menu-button.svelte";

	export let value: { [key: string]: boolean };
	export let toAdd: string[] = [];
	export let toRemove: string[] = [];
	export let toRename: string[] = [];
	export let emote: Emote;
	export let alias: string;
	export let disabled: boolean = false;
	export let hideEmoteDialogMode: () => void;

	let searchQuery = "";
	let favEmoteSets: string[] = [];
	let moreMenuDropdown: ReturnType<typeof DropDown>;
	let favEmoteSetsAction: "add" | "remove" | null = null;

	if (browser) {
		const stored = window.localStorage.getItem("fav-emote-sets");
		if (stored) {
			try {
				favEmoteSets = JSON.parse(stored);
			} catch {
				favEmoteSets = [];
			}
		}
	}

	function handleEmoteSetIDInFavs(id: string) {
		if (favEmoteSets.includes(id)) {
			favEmoteSets = favEmoteSets.filter((setId) => setId !== id);
		} else {
			favEmoteSets = [...favEmoteSets, id];
		}
		window.localStorage.setItem("fav-emote-sets", JSON.stringify(favEmoteSets));
	}

	function groupByOwnerId(sets: EmoteSet[]) {
		const init: { [key: string]: EmoteSet[] } = {};

		const defaultSet =
			$defaultEmoteSet && $editableEmoteSets.find((set) => set.id === $defaultEmoteSet);
		if (defaultSet && defaultSet.owner?.id) {
			init[defaultSet.owner.id] = [];
		}

		if ($user) {
			init[$user.id] = [];
		}

		return sets.reduce((grouped, set) => {
			if (set.owner) {
				(grouped[set.owner.id] = grouped[set.owner.id] || []).push(set);
			}
			return grouped;
		}, init);
	}

	function searchFilter(sets: EmoteSet[], query: string) {
		if (query.length === 0) {
			return sets;
		}

		return sets.filter((set) => {
			return (
				set.name.toLowerCase().includes(query.toLowerCase()) ||
				set.owner?.mainConnection?.platformDisplayName?.toLowerCase().includes(query.toLowerCase())
			);
		});
	}

	$: editableSets = $editableEmoteSets
		? groupByOwnerId(searchFilter($editableEmoteSets, searchQuery))
		: {};

	function onExpand(ownerId: string, expanded: boolean) {
		if (!browser || searchQuery.length !== 0) {
			return;
		}

		window.localStorage.setItem(`emote-set-picker-${ownerId}`, JSON.stringify(expanded));
	}

	function loadExpanded(ownerId: string): boolean | undefined {
		if (!browser) {
			return undefined;
		}

		const value = window.localStorage.getItem(`emote-set-picker-${ownerId}`);
		return value ? (JSON.parse(value) ?? undefined) : undefined;
	}

	function isDisabled(set: EmoteSet) {
		if (disabled) {
			return true;
		}

		return title(set) !== undefined;
	}

	function title(set: EmoteSet) {
		if (!value[set.id] && set.capacity && set.emotes.totalCount >= set.capacity) {
			return "Capacity Reached";
		}

		return undefined;
	}
</script>

<div class="picker">
	<TextInput placeholder={$t("labels.search_emote_set")} bind:value={searchQuery}>
		{#snippet icon()}
			<MagnifyingGlass />
		{/snippet}
	</TextInput>
	<!-- Handle fav emote sets -->
	<Expandable
		title={$t("dialogs.emote_set.favourite_sets.title")}
		expanded={searchQuery.length !== 0 ||
			favEmoteSets.some((id) =>
				Object.keys(editableSets).some((ownerId) =>
					editableSets[ownerId].some((set) => set.id === id),
				),
			)}
		onexpand={(expanded) => onExpand("fav", expanded)}
	>
		<DropDown bind:this={moreMenuDropdown} align="left">
			<Button secondary>
				{$t("labels.actions")}
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>

			{#snippet dropdown()}
				<div class="dropdown">
					<MenuButton
						onclick={() => {
							favEmoteSetsAction = "add";
							// Set all favorite sets to true
							favEmoteSets.forEach((id) => {
								if (value[id] !== undefined) value[id] = true;
							});
							moreMenuDropdown?.close();
						}}
					>
						<Plus />
						{$t("dialogs.emote_set.favourite_sets.add")}
					</MenuButton>

					<MenuButton
						onclick={() => {
							favEmoteSetsAction = "remove";
							// Set all favorite sets to false
							favEmoteSets.forEach((id) => {
								if (value[id] !== undefined) value[id] = false;
							});
							moreMenuDropdown?.close();
						}}
					>
						<Minus />
						{$t("dialogs.emote_set.favourite_sets.remove")}
					</MenuButton>
				</div>
			{/snippet}
		</DropDown>
		{#each favEmoteSets as favSetId}
			{#each Object.keys(editableSets) as ownerId}
				{#each editableSets[ownerId] as set (set.id)}
					{#if set.id === favSetId}
						{#snippet pickerLeftLabel()}
							<div class="emote-set">
								{#if toAdd.includes(set.id)}
									<Plus />
								{/if}
								{#if toRemove.includes(set.id)}
									<Minus />
								{/if}
								<p title={set.name}>
									{set.name.length > 13 ? set.name.slice(0, 13) + "…" : set.name}
								</p>
								<p style="opacity: 0.4;">({set.owner?.mainConnection?.platformDisplayName})</p>
								<Flags
									flags={[
										`${set.emotes.totalCount}/${set.capacity}`,
										...($defaultEmoteSet === set.id ? ["default"] : []),
									]}
								/>
							</div>
						{/snippet}
						{#if value && value[set.id] !== undefined}
							<Checkbox
								option
								favSetId={set.id}
								isFavAlready={favEmoteSets.includes(set.id)}
								{handleEmoteSetIDInFavs}
								leftLabel={pickerLeftLabel}
								disabled={isDisabled(set)}
								bind:value={value[set.id]}
								style={`flex: 1;border-color: ${toAdd.includes(set.id) ? "var(--approve)" : toRemove.includes(set.id) ? "var(--danger)" : undefined}`}
								title={title(set)}
								favIcon={true}
							/>
						{:else}
							<div class="placeholder">
								<Spinner />
							</div>
						{/if}
					{/if}
				{/each}
			{/each}
		{/each}
	</Expandable>

	<!-- All emote sets -->

	{#each Object.keys(editableSets) as ownerId}
		<Expandable
			title={editableSets[ownerId][0]?.owner?.mainConnection?.platformDisplayName ?? "Emote Sets"}
			expanded={searchQuery.length !== 0 || (loadExpanded(ownerId) ?? ownerId === $user?.id)}
			onexpand={(expanded) => onExpand(ownerId, expanded)}
		>
			{#each editableSets[ownerId] as set}
				{#snippet pickerLeftLabel()}
					<div class="emote-set">
						{#if toAdd.includes(set.id)}
							<Plus />
						{/if}
						{#if toRemove.includes(set.id)}
							<Minus />
						{/if}
						{#if toRename.includes(set.id)}
							<Plus />
						{/if}
						<p title={set.name}>
							{set.name.length > 13 ? set.name.slice(0, 13) + "…" : set.name}
						</p>
						<Flags
							flags={[
								`${set.emotes.totalCount}/${set.capacity}`,
								...($defaultEmoteSet === set.id ? ["default"] : []),
							]}
						/>
					</div>
				{/snippet}
				{#if value && value[set.id] !== undefined}
					<Checkbox
						option
						favSetId={set.id}
						isFavAlready={favEmoteSets.includes(set.id)}
						{handleEmoteSetIDInFavs}
						leftLabel={pickerLeftLabel}
						disabled={isDisabled(set)}
						bind:value={value[set.id]}
						style={`flex: 1;border-color: ${toAdd.includes(set.id) ? "var(--approve)" : toRemove.includes(set.id) ? "var(--danger)" : undefined}`}
						title={title(set)}
						favIcon={true}
					/>
				{:else}
					<div class="placeholder">
						<Spinner />
					</div>
				{/if}
			{/each}
		</Expandable>
	{/each}
</div>

<style lang="scss">
	.emote-alias {
		font-size: 0.85rem;
		color: var(--text-light);
		margin-left: 0.25rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 6rem;
		vertical-align: bottom;
	}
	.dropdown {
		display: flex;
		flex-direction: column;
	}
	.emote {
		width: 100%;
		max-width: 10rem;
		aspect-ratio: 1 / 1;
		border: 1px solid #e0823d80;
		border-radius: 0.25rem;
		&:hover,
		&:focus-visible {
			border-color: #e0823d;
		}

		& > :global(picture) {
			flex-grow: 1;
			margin-bottom: 0.5rem;
			line-height: 0;

			width: 100%;
			max-width: 60%;
			max-height: 50%;
		}
	}
	.existing-emote {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 0.3rem;
		p {
			font-size: 0.875rem;
			font-weight: 500;
			color: var(--text-light);
		}
	}
	.preview {
		flex-grow: 1;
		align-self: center;
		width: 1rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.picker {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.emote-set {
		font-size: 0.875rem;
		font-weight: 500;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.placeholder {
		display: flex;
		justify-content: center;
		align-items: center;

		padding: 0.97rem 0.75rem;
		border-radius: 0.5rem;
		background-color: var(--bg-medium);
		border: 1px solid transparent;
	}
</style>
