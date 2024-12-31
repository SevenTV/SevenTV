<script lang="ts">
	import { graphql } from "$/gql";
	import { EntitlementNodeTypeInput, type EntitlementNodeInput } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Select from "../input/select.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import type { DialogMode } from "./dialog.svelte";
	import Dialog from "./dialog.svelte";

	interface Props {
		mode: DialogMode;
		from: EntitlementNodeInput;
		fromName: string;
	}

	let { mode = $bindable(), from, fromName }: Props = $props();

	let type = $state<EntitlementNodeTypeInput | "">("");
	let id = $state<string>();

	let loading = $state(false);

	async function submit() {
		if (!type || !id) return;

		loading = true;

		await gqlClient().mutation(
			graphql(`
				mutation AdminAssignEnititlement($from: EntitlementNodeInput!, $to: EntitlementNodeInput!) {
					entitlementEdges {
						create(from: $from, to: $to) {
							__typename
						}
					}
				}
			`),
			{
				from,
				to: {
					type,
					id,
				},
			},
		);

		loading = false;
		mode = "hidden";
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>Create Entitlements</h1>
		<hr />
		<p>Assign entitlements to <b>{from.type.replace("_", " ")} {fromName}</b></p>
		<label>
			Type
			<Select
				bind:selected={type}
				options={[
					{ value: EntitlementNodeTypeInput.Role, label: "Role" },
					{ value: EntitlementNodeTypeInput.Badge, label: "Badge" },
					{ value: EntitlementNodeTypeInput.Paint, label: "Paint" },
					{ value: EntitlementNodeTypeInput.EmoteSet, label: "Emote Set" },
				]}
			/>
		</label>
		<TextInput bind:value={id} placeholder="ID">
			{type.replace("_", " ")} ID
		</TextInput>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		<div class="buttons">
			<Button
				primary
				icon={loading ? loadingSpinner : undefined}
				onclick={submit}
				disabled={loading || !type || !id}
			>
				Create Entitlements
			</Button>
			<Button secondary onclick={() => (mode = "hidden")}>Cancel</Button>
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

	.buttons {
		display: flex;
		gap: 1rem;
	}
</style>
