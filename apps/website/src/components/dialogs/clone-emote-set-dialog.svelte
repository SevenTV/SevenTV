<script lang="ts">
	import { goto } from "$app/navigation";
	import { user } from "$/lib/auth";
	import { gqlClient } from "$/lib/gql";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";

	interface Props {
		mode: DialogMode;
		sourceId: string;
		sourceName: string;
	}

	let { mode = $bindable("hidden"), sourceId, sourceName }: Props = $props();

	let copyMode: "clone" | "merge" = $state("clone");
	let newName = $state(`${sourceName} (Copy)`);
	let overrideConflicts = $state(false);
	let targetSetId = $state("");
	let loading = $state(false);
	let error = $state("");
	let result: { copied: number; skipped: number } | null = $state(null);

	async function gql(query: string, variables: Record<string, unknown>) {
		if (!$user) {
			throw new Error("You need to be logged in to copy or clone Emote Sets.");
		}

		const res = await gqlClient()
			.mutation(query as any, variables)
			.toPromise();

		if (res.error) {
			if (res.error.networkError) {
				throw new Error("Network error. Please check your connection and try again.");
			}
			if (res.error.graphQLErrors.length > 0) {
				throw new Error(res.error.graphQLErrors[0].message);
			}
			throw new Error("An unknown error occurred.");
		}

		return res.data;
	}

	const CLONE_MUTATION = `
		mutation CloneEmoteSet(
			$sourceId: EmoteSetId!
			$name: String!
			$overrideConflicts: Boolean
		) {
			emoteSet {
				cloneEmoteSet(
					id: $sourceId
					name: $name
					tags: []
					overrideConflicts: $overrideConflicts
				) {
					emoteSet { id name }
					copied
					entries { status }
				}
			}
		}
	`;

	const MERGE_MUTATION = `
		mutation MergeEmoteSet(
			$targetId: EmoteSetId!
			$sourceId: EmoteSetId!
			$overrideConflicts: Boolean
		) {
			emoteSet {
				emoteSet(id: $targetId) {
					copyFrom(
						sourceId: $sourceId
						overrideConflicts: $overrideConflicts
					) {
						emoteSet { id name }
						copied
						entries { status }
					}
				}
			}
		}
	`;

	function mapApiError(message: string, isMerge: boolean): string {
		const lower = message.toLowerCase();
		if (
			lower.includes("lacking privileges") ||
			lower.includes("forbidden") ||
			lower.includes("permission")
		) {
			if (isMerge) {
				return "You don\'t have permission to add emotes to this set. You need to be an editor of the target set\'s owner.";
			}
			return "You don\'t have permission to perform this action.";
		}
		if (lower.includes("not found")) {
			return "The target Emote Set could not be found. Please check the ID.";
		}
		if (lower.includes("capacity")) {
			return "The target Emote Set is at full capacity.";
		}
		if (lower.includes("identical")) {
			return "Source and target Emote Set cannot be the same.";
		}
		return message;
	}

	async function submit() {
		if (loading) return;
		loading = true;
		error = "";
		result = null;

		try {
			if (copyMode === "clone") {
				if (!newName.trim()) {
					error = "Please name your new Emote Set!";
					return;
				}
				const data = await gql(CLONE_MUTATION, {
					sourceId,
					name: newName.trim(),
					overrideConflicts,
				});
				const res = data.emoteSet.cloneEmoteSet;
				const skipped = res.entries.filter((e: { status: string }) =>
					e.status.startsWith("SKIPPED"),
				).length;
				result = { copied: res.copied, skipped };
				setTimeout(() => goto(`/emote-sets/${res.emoteSet.id}`), 1500);
			} else {
				if (!targetSetId.trim()) {
					error = "Please enter the ID of the targeted Emote Set.";
					return;
				}
				const data = await gql(MERGE_MUTATION, {
					targetId: targetSetId.trim(),
					sourceId,
					overrideConflicts,
				});
				const res = data.emoteSet.emoteSet.copyFrom;
				const skipped = res.entries.filter((e: { status: string }) =>
					e.status.startsWith("SKIPPED"),
				).length;
				result = { copied: res.copied, skipped };
			}
		} catch (e) {
			error = e instanceof Error ? e.message : "Unknown Error";
		} finally {
			loading = false;
		}
	}

	function close() {
		mode = "hidden";
		error = "";
		result = null;
		newName = `${sourceName} (Copy)`;
		targetSetId = "";
		overrideConflicts = false;
		copyMode = "clone";
	}
</script>

<Dialog bind:mode>
	<form
		class="layout"
		onsubmit={(e) => {
			e.preventDefault();
			submit();
		}}
	>
		<h1>Copy Emote Set</h1>
		<hr />

		<!-- Mode Tabs -->
		<div class="tabs" role="tablist">
			<button
				type="button"
				role="tab"
				aria-selected={copyMode === "clone"}
				class:active={copyMode === "clone"}
				onclick={() => (copyMode = "clone")}
			>
				Clone as new set
			</button>
			<!-- Merge-Tab only  -->
			{#if $user}
				<button
					type="button"
					role="tab"
					aria-selected={copyMode === "merge"}
					class:active={copyMode === "merge"}
					onclick={() => (copyMode = "merge")}
				>
					Add to existing set
				</button>
			{/if}
		</div>

		{#if copyMode === "clone" || !$user}
			<TextInput
				placeholder="e.g. My Set (Copy)"
				bind:value={newName}
				disabled={loading || !!result}
			>
				<span class="label">Name of new set</span>
			</TextInput>
		{:else}
			<TextInput placeholder="60b0a6c8..." bind:value={targetSetId} disabled={loading || !!result}>
				<span class="label">ID of targeted set</span>
			</TextInput>
		{/if}

		<label class="checkbox-field">
			<input type="checkbox" bind:checked={overrideConflicts} disabled={loading || !!result} />
			<span class="label">Override alias conflicts</span>
		</label>

		{#if error}
			<p class="error">{error}</p>
		{/if}

		{#if result}
			<p class="success">
				✓ {result.copied} Emote{result.copied !== 1 ? "s" : ""} copied, {result.skipped} skipped.
				{#if copyMode === "clone"}Forwarding…{/if}
			</p>
		{/if}

		<div class="buttons">
			<Button secondary onclick={close} disabled={loading || !!result}>Cancel</Button>
			{#if !result}
				{#snippet loadingSpinner()}
					<Spinner />
				{/snippet}
				<Button
					primary
					submit
					onclick={submit}
					disabled={loading}
					icon={loading ? loadingSpinner : undefined}
				>
					{#if copyMode === "clone"}
						Clone & Copy
					{:else}
						Add to Set
					{/if}
				</Button>
			{/if}
		</div>
	</form>
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

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.tabs {
		display: flex;
		border-bottom: 1px solid var(--border-color);
		margin: -0.25rem -1rem 0;
		padding: 0 0.25rem;
	}

	.tabs button {
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		padding: 0.5rem 0.75rem;
		font-size: 0.875rem;
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s;
	}

	.tabs button.active,
	.tabs button:hover {
		color: var(--accent);
		border-bottom-color: var(--accent);
	}

	.checkbox-field {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		cursor: pointer;
		user-select: none;
	}

	.error {
		background: rgba(255, 80, 80, 0.15);
		border: 1px solid rgba(255, 80, 80, 0.4);
		border-radius: 4px;
		color: var(--danger);
		font-size: 0.85rem;
		margin: 0;
		padding: 0.6rem 0.75rem;
	}

	.success {
		background: rgba(0, 220, 120, 0.12);
		border: 1px solid rgba(0, 220, 120, 0.35);
		border-radius: 4px;
		color: #4ade80;
		font-size: 0.875rem;
		margin: 0;
		padding: 0.6rem 0.75rem;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 0.5rem;
	}
</style>
