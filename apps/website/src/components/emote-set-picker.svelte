<script lang="ts">
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import { type Emote, type EmoteSet } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import Spinner from "./spinner.svelte";
	import { browser } from "$app/environment";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { MagnifyingGlass, Minus, PencilSimple, Plus, Warning } from "phosphor-svelte";
	import TextInput from "./input/text-input.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";

	interface Props {
		value: { [key: string]: boolean };
		toAdd?: string[];
		toRemove?: string[];
		toRename?: string[];
		emote: Emote;
		alias: string;
		disabled?: boolean;
	}

	let {
		value = $bindable(),
		toAdd = [],
		toRemove = [],
		toRename = [],
		emote,
		alias,
		disabled = false,
	}: Props = $props();

	let searchQuery = $state("");

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

	let editableSets = $derived(
		$editableEmoteSets ? groupByOwnerId(searchFilter($editableEmoteSets, searchQuery)) : {},
	);

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
		return set.emotes.items.some((e) => e.alias === alias && e.id !== emote.id);
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

<div class="picker">
	<TextInput placeholder="Search Emote Set" bind:value={searchQuery}>
		{#snippet icon()}
			<MagnifyingGlass />
		{/snippet}
	</TextInput>
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
						{set.name}
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
						leftLabel={pickerLeftLabel}
						disabled={isDisabled(set)}
						bind:value={value[set.id]}
						style={`border-color: ${toAdd.includes(set.id) ? "var(--approve)" : toRemove.includes(set.id) ? "var(--danger)" : toRename.includes(set.id) ? "var(--rename)" : undefined}`}
						title={title(set)}
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
