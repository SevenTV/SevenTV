<script lang="ts">
	import { graphql } from "$/gql";
	import { EntitlementNodeTypeInput, type EntitlementNodeInput } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Radio from "../input/radio.svelte";
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
	let id = $state<string>("");

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

	async function queryRoles() {
		const res = await gqlClient().query(
			graphql(`
				query AdminRoles {
					roles {
						roles {
							id
							name
						}
					}
				}
			`),
			{},
		);

		if (!res.data) throw res.error?.message;

		return res.data.roles.roles;
	}

	let roles = $derived(type === EntitlementNodeTypeInput.Role ? queryRoles() : []);

	async function queryBadges() {
		const res = await gqlClient().query(
			graphql(`
				query AdminBadges {
					badges {
						badges {
							id
							name
						}
					}
				}
			`),
			{},
		);

		if (!res.data) throw res.error?.message;

		return res.data.badges.badges;
	}

	let badges = $derived(type === EntitlementNodeTypeInput.Badge ? queryBadges() : []);

	async function queryPaints() {
		const res = await gqlClient().query(
			graphql(`
				query AdminPaints {
					paints {
						paints {
							id
							name
						}
					}
				}
			`),
			{},
		);

		if (!res.data) throw res.error?.message;

		return res.data.paints.paints;
	}

	let paints = $derived(type === EntitlementNodeTypeInput.Paint ? queryPaints() : []);
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>Create Entitlements</h1>
		<hr />
		<p>Assign entitlements to <b>{from.type.replace("_", " ")} {fromName}</b></p>
		<div class="types">
			Type
			<Radio bind:group={type} name="to-type" value={EntitlementNodeTypeInput.Role}>
				Role
			</Radio>
			<Radio bind:group={type} name="to-type" value={EntitlementNodeTypeInput.Badge}>
				Badge
			</Radio>
			<Radio bind:group={type} name="to-type" value={EntitlementNodeTypeInput.Paint}>
				Paint
			</Radio>
			<Radio bind:group={type} name="to-type" value={EntitlementNodeTypeInput.EmoteSet}>
				Emote Set
			</Radio>
		</div>
		{#if type}
			{#if type === EntitlementNodeTypeInput.Role}
				<label>
					Role
					{#await roles}
						<Spinner />
					{:then roles}
						<Select
							bind:selected={id}
							options={roles.map((r) => {
								return { value: r.id, label: r.name };
							})}
						/>
					{/await}
				</label>
			{:else if type === EntitlementNodeTypeInput.Badge}
				<label>
					Badge
					{#await badges}
						<Spinner />
					{:then badges}
						<Select
							bind:selected={id}
							options={badges.map((b) => {
								return { value: b.id, label: b.name };
							})}
						/>
					{/await}
				</label>
			{:else if type === EntitlementNodeTypeInput.Paint}
				<label>
					Paint
					{#await paints}
						<Spinner />
					{:then paints}
						<Select
							bind:selected={id}
							options={paints.map((p) => {
								return { value: p.id, label: p.name };
							})}
						/>
					{/await}
				</label>
			{:else}
				<TextInput bind:value={id} placeholder="ID">
					{type.replace("_", " ")} ID
				</TextInput>
			{/if}
		{/if}
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

	.types {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	:global(.select:has(.dropped)) {
		margin-bottom: 15rem;
	}

	.buttons {
		display: flex;
		gap: 1rem;
	}
</style>
