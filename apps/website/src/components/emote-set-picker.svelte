<script lang="ts">
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import Radio from "./input/radio.svelte";
	import { type EmoteSet } from "$/gql/graphql";
	import { user } from "$/lib/auth";

	let { radioName }: { radioName?: string } = $props();

	function groupByOwnerId(sets: EmoteSet[]) {
		return sets.reduce((grouped, set) => {
			if (set.owner) {
				(grouped[set.owner.id] = grouped[set.owner.id] || []).push(set);
			}
			return grouped;
		}, {} as { [key: string]: EmoteSet[] });
	}

	let editableSets = $derived($user?.editableEmoteSets ? groupByOwnerId($user.editableEmoteSets) : {});
</script>

{#each Object.keys(editableSets) as ownerId}
	<Expandable title={editableSets[ownerId][0]?.owner?.mainConnection?.platformDisplayName ?? "Emote Sets"}>
		{#each editableSets[ownerId] as set}
			{#snippet pickerLeftLabel()}
				<div class="emote-set">
					{set.name}
					<Flags flags={[`${set.emotes.totalCount}/${set.capacity}`, "default"]} />
				</div>
			{/snippet}
			{#if radioName}
				<Radio name={radioName} option leftLabel={pickerLeftLabel} />
			{:else}
				<Checkbox option leftLabel={pickerLeftLabel} />
			{/if}
		{/each}
	</Expandable>
{/each}

<style lang="scss">
	.emote-set {
		font-size: 0.875rem;
		font-weight: 500;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
