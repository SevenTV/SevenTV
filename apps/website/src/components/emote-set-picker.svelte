<script lang="ts">
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import Radio from "./input/radio.svelte";
	import { type EmoteSet } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import Spinner from "./spinner.svelte";
	import { browser } from "$app/environment";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";

	interface Props {
		value: { [key: string]: boolean };
		radioName?: string;
		radioValue?: string;
		disabled?: boolean;
		checkCapacity?: boolean;
	}

	let {
		value = $bindable(),
		radioName,
		radioValue = $bindable(),
		disabled = false,
		checkCapacity = true,
	}: Props = $props();

	function groupByOwnerId(sets: EmoteSet[]) {
		const init: { [key: string]: EmoteSet[] } = {};

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

	let editableSets = $derived(
		$user?.editableEmoteSets ? groupByOwnerId($user.editableEmoteSets) : {},
	);

	function onExpand(ownerId: string, expanded: boolean) {
		if (!browser) {
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
</script>

<div class="picker">
	{#each Object.keys(editableSets) as ownerId}
		<Expandable
			title={editableSets[ownerId][0]?.owner?.mainConnection?.platformDisplayName ?? "Emote Sets"}
			expanded={loadExpanded(ownerId) ?? ownerId === $user?.id}
			onexpand={(expanded) => onExpand(ownerId, expanded)}
		>
			{#each editableSets[ownerId] as set}
				{#snippet pickerLeftLabel()}
					<div class="emote-set">
						{set.name}
						<Flags
							flags={[
								`${set.emotes.totalCount}/${set.capacity}`,
								...($defaultEmoteSet === set.id ? ["default"] : []),
							]}
						/>
					</div>
				{/snippet}
				{#if (value && value[set.id] !== undefined) || radioName !== undefined}
					{#if radioName}
						<Radio
							name={radioName}
							value={set.id}
							bind:group={radioValue}
							option
							leftLabel={pickerLeftLabel}
							disabled={disabled ||
								(checkCapacity && (set.capacity ? set.emotes.totalCount >= set.capacity : false))}
						/>
					{:else}
						<Checkbox
							option
							leftLabel={pickerLeftLabel}
							disabled={disabled ||
								(checkCapacity && (set.capacity ? set.emotes.totalCount >= set.capacity : false))}
							bind:value={value[set.id]}
						/>
					{/if}
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
