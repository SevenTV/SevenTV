<script lang="ts">
	import { graphql } from "$/gql";
	import { type EntitlementNodeInput } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import Spinner from "../spinner.svelte";
	import type { DialogMode } from "./dialog.svelte";
	import Dialog from "./dialog.svelte";

	interface Props {
		mode: DialogMode;
		from: EntitlementNodeInput;
		fromName: string;
	}

	let { mode = $bindable(), from, fromName }: Props = $props();

	async function query(from: EntitlementNodeInput) {
		const res = await gqlClient().query(
			graphql(`
				query AdminViewEnititlement($from: EntitlementNodeInput!) {
					entitlements {
						traverse(from: $from) {
							nodes {
								__typename
								... on EntitlementNodeUser {
									user {
										mainConnection {
											platformDisplayName
										}
									}
								}
								... on EntitlementNodeRole {
									role {
										name
									}
								}
								... on EntitlementNodeBadge {
									badge {
										name
									}
								}
								... on EntitlementNodePaint {
									paint {
										name
									}
								}
								... on EntitlementNodeEmoteSet {
									emoteSet {
										name
									}
								}
								... on EntitlementNodeProduct {
									productId
								}
								... on EntitlementNodeSubscriptionBenefit {
									subscriptionBenefit {
										name
									}
								}
								... on EntitlementNodeSubscription {
									subscriptionId {
										userId
										productId
									}
								}
								... on EntitlementNodeSpecialEvent {
									specialEvent {
										name
									}
								}
							}
						}
					}
				}
			`),
			{
				from,
			},
		);

		if (!res.data) throw res.error?.message;

		return res.data.entitlements.traverse.nodes;
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>View Entitlements</h1>
		<hr />
		<p>Showing entitlements assigned to <b>{from.type.replace("_", " ")} {fromName}</b></p>
		{#await query(from)}
			<Spinner />
		{:then nodes}
			<ul>
				{#each nodes as node}
					<li>
						{#if node.__typename === "EntitlementNodeUser"}
							User: {node.user?.mainConnection?.platformDisplayName}
						{:else if node.__typename === "EntitlementNodeRole"}
							Role: {node.role?.name}
						{:else if node.__typename === "EntitlementNodeBadge"}
							Badge: {node.badge?.name}
						{:else if node.__typename === "EntitlementNodePaint"}
							Paint: {node.paint?.name}
						{:else if node.__typename === "EntitlementNodeEmoteSet"}
							Emote Set: {node.emoteSet?.name}
						{:else if node.__typename === "EntitlementNodeProduct"}
							Product: {node.productId}
						{:else if node.__typename === "EntitlementNodeSubscriptionBenefit"}
							Subscription Benefit: {node.subscriptionBenefit?.name}
						{:else if node.__typename === "EntitlementNodeSubscription"}
							Subscription: {node.subscriptionId.userId}:{node.subscriptionId.productId}
						{:else if node.__typename === "EntitlementNodeSpecialEvent"}
							Special Event: {node.specialEvent?.name}
						{/if}
					</li>
				{/each}
			</ul>
		{/await}
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	ul {
		list-style: inside;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}
</style>
