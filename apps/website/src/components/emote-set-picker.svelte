<script lang="ts">
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import { type Emote, type EmoteSet, type EmoteSetEmote } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import Spinner from "./spinner.svelte";
	import { browser } from "$app/environment";
	import EmotePreview from "../components/emote-preview.svelte";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import MultipleEmoteAliasesManager from "./dialogs/multiple-emote-aliases-manager.svelte";
	import DropDown from "../components/drop-down.svelte";
	import { type DialogMode } from "./dialogs/dialog.svelte";
	import {
		CaretDown,
		DotsThreeVertical,
		GitBranch,
		MagnifyingGlass,
		Minus,
		PencilSimple,
		Plus,
		Warning,
	} from "phosphor-svelte";
	import TextInput from "./input/text-input.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";

	import EmoteContextMenu from "./emote-in-set-dialog.svelte";
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

	let menuPosition: { x: number; y: number } | undefined = undefined;
	let searchQuery = "";
	let emoteSetForContextMenu: EmoteSet | null = null;
	let emoteForContextMenu: EmoteSetEmote | Emote | undefined = undefined;
	let favEmoteSets: string[] = [];
	let moreMenuDropdown: ReturnType<typeof DropDown>;
	let favEmoteSetsAction: "add" | "remove" | null = null;
	let multipleEmoteAliasesManagerDialogMode: DialogMode = "hidden";
	let multipleEmoteAliasesManagerData: {
		all: EmoteSetEmote[];
		count: number;
		status: boolean;
		emote: Emote;
	} = {
		all: [],
		count: 0,
		status: false,
		emote: emote,
	};
	let emoteSetId: string | undefined = undefined;
	let emoteSetName: string | undefined = undefined;

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

	function toggleEmoteContextMenu(e: MouseEvent) {
		e.preventDefault();
		menuPosition = menuPosition
			? undefined
			: {
					x: e.clientX,
					y: e.clientY,
				};
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

	function isConflictingName(set: EmoteSet) {
		return set.emotes.items.some((e) => e.alias === alias && (e.emote?.id ?? e.id) !== emote.id);
	}

	function isEmoteAlreadyInSet(set: EmoteSet) {
		const matches = set.emotes.items.filter((e) => (e.emote?.id ?? e.id) === emote.id);
		return {
			status: matches.length > 0,
			emote: matches[0],
			count: matches.length,
			all: matches,
		};
	}

	function isEmoteAliasAlreadyInSet(set: EmoteSet) {
		const matches = set.emotes.items.filter((e) => e.alias === alias);
		return {
			status: matches.length > 0,
			emote: matches[0],
			count: matches.length,
			all: matches,
		};
	}

	function title(set: EmoteSet) {
		if (!value[set.id] && set.capacity && set.emotes.totalCount >= set.capacity) {
			return "Capacity Reached";
		}

		if (isConflictingName(set)) {
			return "Conflicting Name";
		}

		return undefined;
	}
</script>

{#if multipleEmoteAliasesManagerDialogMode == "shown"}
	<MultipleEmoteAliasesManager
		bind:mode={multipleEmoteAliasesManagerDialogMode}
		data={emote}
		bind:multipleEmoteAliasesManagerData
		{emoteSetId}
		{emoteSetName}
	/>
{/if}

<EmoteContextMenu
	currentEmoteSelected={emote}
	data={emoteForContextMenu as Emote | EmoteSetEmote}
	{hideEmoteDialogMode}
	{emoteSetForContextMenu}
	bind:position={menuPosition}
/>

<div class="picker">
	<TextInput placeholder={$t("labels.search_emote_set")} bind:value={searchQuery}>
		{#snippet icon()}
			<MagnifyingGlass />
		{/snippet}
	</TextInput>
	<!-- Handle fav emote sets -->
	<Expandable
		title={$t("dialogs.emote_set.favourite_sets.title")}
		expanded={favEmoteSets.some((id) =>
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
								{#if toRename.includes(set.id)}
									<PencilSimple />
								{/if}
								{#if isConflictingName(set)}
									<Warning />
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
								{#if isEmoteAlreadyInSet(set).status}
									{#if isEmoteAlreadyInSet(set).count > 1}
										<div class="existing-emote" title="Manage multiple aliases">
											<GitBranch
												size={24}
												style="z-index: 5;vertical-align: middle; margin-right: 0.25rem; cursor: pointer; fill: var(--primary);"
												onclick={(e) => {
													e.stopPropagation();
													multipleEmoteAliasesManagerData = {
														...isEmoteAlreadyInSet(set),
														emote: emote,
													};
													multipleEmoteAliasesManagerDialogMode = "shown";
													emoteSetId = set.id;
													emoteSetName = set.name;
												}}
											/>
											{isEmoteAlreadyInSet(set).count}
										</div>
									{:else}
										<div class="existing-emote">
											<div class="preview">
												<EmotePreview data={emote} emoteOnly />
											</div>
											<p class="emote-alias">{isEmoteAlreadyInSet(set).emote?.alias}</p>
											<DotsThreeVertical
												size={24}
												style="cursor: pointer; fill: var(--primary); z-index: 5;"
												onclick={(e) => {
													e.stopPropagation();
													toggleEmoteContextMenu(e);
													let emoteForMenu = isEmoteAlreadyInSet(set).emote;
													if (!emoteForMenu) {
														emoteForMenu = isEmoteAliasAlreadyInSet(set).emote;
													}
													emoteForContextMenu = emoteForMenu;
													emoteSetForContextMenu = set;
												}}
											/>
										</div>
										<div class="existing-emote" title="Manage multiple aliases">
											<GitBranch
												size={24}
												style="z-index: 5;vertical-align: middle; margin-right: 0.25rem; cursor: pointer; fill: var(--primary);"
												onclick={(e) => {
													e.stopPropagation();
													multipleEmoteAliasesManagerData = {
														...isEmoteAlreadyInSet(set),
														emote: emote,
													};
													multipleEmoteAliasesManagerDialogMode = "shown";
													emoteSetId = set.id;
													emoteSetName = set.name;
												}}
											/>
											{isEmoteAlreadyInSet(set).count}
										</div>
									{/if}
								{:else if isEmoteAliasAlreadyInSet(set).status && isEmoteAliasAlreadyInSet(set).emote?.emote?.id}
									<div class="existing-emote">
										<div class="preview">
											<img
												class="emote"
												src={`https://cdn.7tv.app/emote/${isEmoteAliasAlreadyInSet(set).emote?.emote?.id}/1x.webp`}
												alt={isEmoteAliasAlreadyInSet(set).emote?.alias}
											/>
										</div>
										<p class="emote-alias">
											{isEmoteAliasAlreadyInSet(set).emote?.alias}
										</p>
										<DotsThreeVertical
											size={24}
											style="cursor: pointer; fill: var(--primary); z-index: 5;"
											onclick={(e) => {
												e.stopPropagation();
												toggleEmoteContextMenu(e);
												let emoteForMenu = isEmoteAlreadyInSet(set).emote;
												if (!emoteForMenu) {
													emoteForMenu = isEmoteAliasAlreadyInSet(set).emote;
												}
												emoteForContextMenu = emoteForMenu;
												emoteSetForContextMenu = set;
											}}
										/>
									</div>
									<div class="existing-emote" title="Manage multiple aliases">
										<GitBranch
											size={24}
											style="z-index: 5;vertical-align: middle; margin-right: 0.25rem; cursor: pointer; fill: var(--primary);"
											onclick={(e) => {
												e.stopPropagation();
												multipleEmoteAliasesManagerData = {
													...isEmoteAlreadyInSet(set),
													emote: emote,
												};
												multipleEmoteAliasesManagerDialogMode = "shown";
												emoteSetId = set.id;
												emoteSetName = set.name;
											}}
										/>
										{isEmoteAlreadyInSet(set).count}
									</div>
								{/if}
							</div>
						{/snippet}
						{#if value && value[set.id] !== undefined}
							<Checkbox
								option
								favSetId={set.id}
								isFavAlready={favEmoteSets.includes(set.id)}
								{handleEmoteSetIDInFavs}
								leftLabel={pickerLeftLabel}
								disabled={isEmoteAlreadyInSet(set).count > 1 || isDisabled(set)}
								bind:value={value[set.id]}
								style={`flex: 1;border-color: ${toAdd.includes(set.id) ? "var(--approve)" : toRemove.includes(set.id) ? "var(--danger)" : toRename.includes(set.id) ? "var(--rename)" : undefined}`}
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
							<PencilSimple />
						{/if}
						{#if isConflictingName(set)}
							<Warning />
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
						{#if isEmoteAlreadyInSet(set).status}
							{#if isEmoteAlreadyInSet(set).count > 1}
								<div class="existing-emote" title="Manage multiple aliases">
									<GitBranch
										size={24}
										style="z-index: 5;vertical-align: middle; margin-right: 0.25rem; cursor: pointer; fill: var(--primary);"
										onclick={(e) => {
											e.stopPropagation();
											multipleEmoteAliasesManagerData = {
												...isEmoteAlreadyInSet(set),
												emote: emote,
											};
											multipleEmoteAliasesManagerDialogMode = "shown";
											emoteSetId = set.id;
											emoteSetName = set.name;
										}}
									/>
									{isEmoteAlreadyInSet(set).count}
								</div>
							{:else}
								<div class="existing-emote">
									<div class="preview">
										<EmotePreview data={emote} emoteOnly />
									</div>
									<p class="emote-alias">{isEmoteAlreadyInSet(set).emote?.alias}</p>
									<DotsThreeVertical
										size={24}
										style="cursor: pointer; fill: var(--primary); z-index: 5;"
										onclick={(e) => {
											e.stopPropagation();
											toggleEmoteContextMenu(e);
											let emoteForMenu = isEmoteAlreadyInSet(set).emote;
											if (!emoteForMenu) {
												emoteForMenu = isEmoteAliasAlreadyInSet(set).emote;
											}
											emoteForContextMenu = emoteForMenu;
											emoteSetForContextMenu = set;
										}}
									/>
								</div>
								<div class="existing-emote" title="Manage multiple aliases">
									<GitBranch
										size={24}
										style="z-index: 5;vertical-align: middle; margin-right: 0.25rem; cursor: pointer; fill: var(--primary);"
										onclick={(e) => {
											e.stopPropagation();
											multipleEmoteAliasesManagerData = {
												...isEmoteAlreadyInSet(set),
												emote: emote,
											};
											multipleEmoteAliasesManagerDialogMode = "shown";
											emoteSetId = set.id;
											emoteSetName = set.name;
										}}
									/>
									{isEmoteAlreadyInSet(set).count}
								</div>
							{/if}
						{:else if isEmoteAliasAlreadyInSet(set).status && isEmoteAliasAlreadyInSet(set).emote?.emote?.id}
							<div class="existing-emote">
								<div class="preview">
									<img
										class="emote"
										src={`https://cdn.7tv.app/emote/${isEmoteAliasAlreadyInSet(set).emote?.emote?.id}/1x.webp`}
										alt={isEmoteAliasAlreadyInSet(set).emote?.alias}
									/>
								</div>
								<p class="emote-alias">
									{isEmoteAliasAlreadyInSet(set).emote?.alias}
								</p>
								<DotsThreeVertical
									size={24}
									style="cursor: pointer; fill: var(--primary); z-index: 5;"
									onclick={(e) => {
										e.stopPropagation();
										toggleEmoteContextMenu(e);
										let emoteForMenu = isEmoteAlreadyInSet(set).emote;
										if (!emoteForMenu) {
											emoteForMenu = isEmoteAliasAlreadyInSet(set).emote;
										}
										emoteForContextMenu = emoteForMenu;
										emoteSetForContextMenu = set;
									}}
								/>
							</div>
							<div class="existing-emote" title="Manage multiple aliases">
								<GitBranch
									size={24}
									style="z-index: 5;vertical-align: middle; margin-right: 0.25rem; cursor: pointer; fill: var(--primary);"
									onclick={(e) => {
										e.stopPropagation();
										multipleEmoteAliasesManagerData = {
											...isEmoteAlreadyInSet(set),
											emote: emote,
										};
										multipleEmoteAliasesManagerDialogMode = "shown";
										emoteSetId = set.id;
										emoteSetName = set.name;
									}}
								/>
								{isEmoteAlreadyInSet(set).count}
							</div>
						{/if}
					</div>
				{/snippet}
				{#if value && value[set.id] !== undefined}
					<Checkbox
						option
						favSetId={set.id}
						isFavAlready={favEmoteSets.includes(set.id)}
						{handleEmoteSetIDInFavs}
						leftLabel={pickerLeftLabel}
						disabled={isEmoteAlreadyInSet(set).count > 1 || isDisabled(set)}
						bind:value={value[set.id]}
						style={`flex: 1;border-color: ${toAdd.includes(set.id) ? "var(--approve)" : toRemove.includes(set.id) ? "var(--danger)" : toRename.includes(set.id) ? "var(--rename)" : undefined}`}
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
