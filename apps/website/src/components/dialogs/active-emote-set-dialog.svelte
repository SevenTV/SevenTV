<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Expandable from "../expandable.svelte";
	import { EmoteSetKind, type EmoteSet, type User } from "$/gql/graphql";
	import { browser } from "$app/environment";
	import Flags from "../flags.svelte";
	import Radio from "../input/radio.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";
	import TextInput from "../input/text-input.svelte";
	import { MagnifyingGlass } from "phosphor-svelte";
	import Spinner from "../spinner.svelte";
	import { setActiveSet } from "$/lib/userMutations";
	import { user } from "$/lib/auth";

	let {
		mode = $bindable("hidden"),
		userData = $bindable(),
	}: { mode: DialogMode; userData: Promise<User | undefined> } = $props();

	let searchQuery = $state("");

	function groupByOwnerId(userId: string, sets: EmoteSet[]) {
		const init: { [key: string]: EmoteSet[] } = {};

		init[userId] = [];

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

	function onExpand(ownerId: string, expanded: boolean) {
		if (!browser || searchQuery.length !== 0) {
			return;
		}

		window.localStorage.setItem(`emote-set-picker-${ownerId}`, JSON.stringify(expanded));
	}

	function loadExpanded(enabledSetId: string | undefined, ownerId: string): boolean | undefined {
		if (!browser) {
			return undefined;
		}

		const defaultSet = $editableEmoteSets.find((set) => set.id === enabledSetId);

		if (defaultSet && defaultSet.owner?.id === ownerId) {
			return true;
		}

		const value = window.localStorage.getItem(`emote-set-picker-${ownerId}`);
		return value ? (JSON.parse(value) ?? undefined) : undefined;
	}

	let activeSet = $state<string>();

	$effect(() => {
		userData.then((user) => {
			if (user) {
				activeSet = user.style.activeEmoteSet?.id;
			}
		});
	});

	$effect(() => {
		if (activeSet) {
			userData.then((user) => {
				if (user && activeSet && activeSet !== user.style.activeEmoteSet?.id) {
					setUserActiveSet(user.id, activeSet);
				}
			});
		}
	});

	function setUserActiveSet(userId: string, setId: string) {
		const promise = setActiveSet(userId, setId);
		promise.then((userData) => {
			if (userData?.id === $user?.id) {
				$user = userData;
			}
		});
		userData = promise;
	}
</script>

<Dialog width={30} bind:mode>
	<div class="layout">
		<h1>Active Emote Set</h1>
		<hr />
		{#if $editableEmoteSets}
			{#await userData}
				<Spinner />
			{:then userData}
				{#if userData}
					{@const editableSets = groupByOwnerId(
						userData.id,
						searchFilter(
							$editableEmoteSets.filter((s) => s.kind === EmoteSetKind.Normal),
							searchQuery,
						),
					)}
					<div class="picker">
						<TextInput placeholder="Search Emote Set" bind:value={searchQuery}>
							{#snippet icon()}
								<MagnifyingGlass />
							{/snippet}
						</TextInput>
						{#each Object.keys(editableSets) as ownerId}
							<Expandable
								title={editableSets[ownerId][0]?.owner?.mainConnection?.platformDisplayName ??
									"Emote Sets"}
								expanded={loadExpanded(userData.style.activeEmoteSet?.id, ownerId) ??
									ownerId === userData.id}
								onexpand={(expanded) => onExpand(ownerId, expanded)}
							>
								{#each editableSets[ownerId] as set}
									{#snippet pickerLeftLabel()}
										<div class="emote-set">
											{set.name}
											<Flags
												flags={[
													`${set.emotes.totalCount}/${set.capacity}`,
													...(userData.style.activeEmoteSet?.id === set.id ? ["active"] : []),
												]}
											/>
										</div>
									{/snippet}
									<Radio
										name="active-set"
										value={set.id}
										bind:group={activeSet}
										option
										leftLabel={pickerLeftLabel}
									/>
								{/each}
							</Expandable>
						{/each}
					</div>
				{/if}
			{/await}
		{:else}
			<Spinner />
		{/if}
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
