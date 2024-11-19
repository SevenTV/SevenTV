<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { t } from "svelte-i18n";
	import Expandable from "../expandable.svelte";
	import type { EmoteSet } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import { browser } from "$app/environment";
	import Flags from "../flags.svelte";
	import Radio from "../input/radio.svelte";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();

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

<Dialog width={30} bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.default_set.title")}</h1>
		<hr />
		<div class="picker">
			{#each Object.keys(editableSets) as ownerId}
				<Expandable
					title={editableSets[ownerId][0]?.owner?.mainConnection?.platformDisplayName ??
						"Emote Sets"}
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
						<Radio
							name="default-set"
							value={set.id}
							bind:group={$defaultEmoteSet}
							option
							leftLabel={pickerLeftLabel}
						/>
					{/each}
				</Expandable>
			{/each}
		</div>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.picker {
		display: flex;
		flex-direction: column;
		gap: 1rem;

		.emote-set {
			font-size: 0.875rem;
			font-weight: 500;

			display: flex;
			align-items: center;
			gap: 0.5rem;
		}
	}
</style>
