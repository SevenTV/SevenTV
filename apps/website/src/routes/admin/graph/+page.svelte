<script lang="ts">
	import { MultiGraph } from "graphology";
	import Sigma from "sigma";
	import ForceSupervisor from "graphology-layout-forceatlas2/worker";
	import TextInput from "$/components/input/text-input.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import type { EntitlementNodeAny } from "$/gql/graphql";
	import Toggle from "$/components/input/toggle.svelte";
	import { t } from "svelte-i18n";
	import { user } from "$/lib/auth";

	// svelte-ignore non_reactive_update
	let sigmaContainer: HTMLDivElement;

	let userId = $state("");

	function nodeToString(node: EntitlementNodeAny) {
		let key = node.__typename + ":";

		switch (node.__typename) {
			case "EntitlementNodeUser":
				key += node.user?.id;
				break;
			case "EntitlementNodeRole":
				key += node.role?.id;
				break;
			case "EntitlementNodeBadge":
				key += node.badge?.id;
				break;
			case "EntitlementNodePaint":
				key += node.paint?.id;
				break;
			case "EntitlementNodeEmoteSet":
				key += node.emoteSet?.id;
				break;
			case "EntitlementNodeProduct":
				key += node.productId;
				break;
			case "EntitlementNodeSubscriptionBenefit":
				key += node.subscriptionBenefit?.id;
				break;
			case "EntitlementNodeSubscription":
				key += node.subscriptionId.userId + ":" + node.subscriptionId.productId;
				break;
			case "EntitlementNodeSpecialEvent":
				key += node.specialEvent?.id;
				break;
		}

		return key;
	}

	async function queryEntitlements(userId: string) {
		if (!userId) {
			return new MultiGraph();
		}

		const res = await gqlClient().query(
			graphql(`
				query RawEntitlements($userId: Id!) {
					users {
						user(id: $userId) {
							rawEntitlements {
								nodes {
									__typename
									... on EntitlementNodeUser {
										user {
											id
											mainConnection {
												platformDisplayName
											}
										}
									}
									... on EntitlementNodeRole {
										role {
											id
											name
										}
									}
									... on EntitlementNodeBadge {
										badge {
											id
											name
										}
									}
									... on EntitlementNodePaint {
										paint {
											id
											name
										}
									}
									... on EntitlementNodeEmoteSet {
										emoteSet {
											id
											name
										}
									}
									... on EntitlementNodeProduct {
										productId
									}
									... on EntitlementNodeSubscriptionBenefit {
										subscriptionBenefit {
											id
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
											id
											name
										}
									}
								}
								edges {
									from {
										__typename
										... on EntitlementNodeUser {
											user {
												id
											}
										}
										... on EntitlementNodeRole {
											role {
												id
											}
										}
										... on EntitlementNodeBadge {
											badge {
												id
											}
										}
										... on EntitlementNodePaint {
											paint {
												id
											}
										}
										... on EntitlementNodeEmoteSet {
											emoteSet {
												id
											}
										}
										... on EntitlementNodeProduct {
											productId
										}
										... on EntitlementNodeSubscriptionBenefit {
											subscriptionBenefit {
												id
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
												id
											}
										}
									}
									to {
										__typename
										... on EntitlementNodeUser {
											user {
												id
											}
										}
										... on EntitlementNodeRole {
											role {
												id
											}
										}
										... on EntitlementNodeBadge {
											badge {
												id
											}
										}
										... on EntitlementNodePaint {
											paint {
												id
											}
										}
										... on EntitlementNodeEmoteSet {
											emoteSet {
												id
											}
										}
										... on EntitlementNodeProduct {
											productId
										}
										... on EntitlementNodeSubscriptionBenefit {
											subscriptionBenefit {
												id
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
												id
											}
										}
									}
								}
							}
						}
					}
				}
			`),
			{ userId },
		);

		if (!res.data?.users.user) {
			throw new Error("User not found");
		}

		const nodes = res.data.users.user.rawEntitlements.nodes.map((node) => {
			let key = nodeToString(node as EntitlementNodeAny);
			let label = "";
			let color = "#000000";

			switch (node.__typename) {
				case "EntitlementNodeUser": {
					label = "User: ";
					label += node.user?.mainConnection?.platformDisplayName ?? key;
					color = "#ff0000";
					break;
				}
				case "EntitlementNodeRole": {
					label = "Role: ";
					label += node.role?.name ?? key;
					color = "#00ff00";
					break;
				}
				case "EntitlementNodeBadge": {
					label = "Badge: ";
					label += node.badge?.name ?? key;
					color = "#0000ff";
					break;
				}
				case "EntitlementNodePaint": {
					label = "Paint: ";
					label += node.paint?.name ?? key;
					color = "#ffff00";
					break;
				}
				case "EntitlementNodeEmoteSet": {
					label = "Emote Set: ";
					label += node.emoteSet?.name ?? key;
					color = "#00ffff";
					break;
				}
				case "EntitlementNodeProduct": {
					label = "Product: ";
					label += node.productId;
					color = "#ff00ff";
					break;
				}
				case "EntitlementNodeSubscriptionBenefit": {
					label = "Subscription Benefit: ";
					label += node.subscriptionBenefit?.name ?? key;
					color = "#ff8000";
					break;
				}
				case "EntitlementNodeSubscription": {
					label = "Subscription: ";
					label += node.subscriptionId.userId + ":" + node.subscriptionId.productId;
					color = "#ff0080";
					break;
				}
				case "EntitlementNodeSpecialEvent": {
					label = "Special Event: ";
					label += node.specialEvent?.name ?? key;
					color = "#80ff00";
					break;
				}
				case "EntitlementNodeGlobalDefaultEntitlementGroup": {
					label = "Global Default Entitlement Group";
					color = "#0080ff";
					break;
				}
			}

			return {
				key,
				attributes: {
					x: Math.random() * 50,
					y: Math.random() * 50,
					size,
					label,
					color,
				},
			};
		});

		const edges = res.data.users.user.rawEntitlements.edges.map((edge) => {
			const source = nodeToString(edge.from as EntitlementNodeAny);
			const target = nodeToString(edge.to as EntitlementNodeAny);

			return {
				source,
				target,
			};
		});

		const graph = new MultiGraph();
		graph.import({
			nodes,
			edges,
		});

		return graph;
	}

	let graph = $derived(queryEntitlements(userId));

	let layoutStarted = $state(true);
	let renderer = $state<Sigma>();
	let layout: ForceSupervisor | undefined = undefined;
	let size = $state(5);

	$effect(() => {
		if (layoutStarted) {
			layout?.start();
		} else {
			layout?.stop();
		}
	});

	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		size;
		renderer?.setSetting("nodeReducer", (_edge, data) => {
			const res = { ...data };
			res.size = size;
			return res;
		});
	});

	$effect(() => {
		graph.then((graph) => {
			size = Math.max(Math.ceil(((-15 + 3) / 350) * graph.size + 15), 0);
			renderer = new Sigma(graph, sigmaContainer);
			layout = new ForceSupervisor(graph);
			layout.start();
			layoutStarted = true;
		});

		return () => {
			layout?.kill();
			renderer?.kill();
		};
	});
</script>

<svelte:head>
	<title>Entitlement Graph - {$t("pages.admin.page_title_suffix")}</title>
</svelte:head>

{#if $user?.permissions.user.manageAny}
	<div class="inputs">
		<TextInput placeholder="User ID" bind:value={userId} />
		<Toggle bind:value={layoutStarted}>Layout</Toggle>
		<label>
			Node Size
			<input type="number" bind:value={size} min="1" max="15" step="1" />
		</label>
	</div>
	<div class="sigma-container light-theme" bind:this={sigmaContainer}></div>
{/if}

<style lang="scss">
	.sigma-container {
		flex-grow: 1;
		background-color: var(--bg-dark);
	}

	.inputs {
		display: flex;
		align-items: center;
		gap: 1rem;
	}
</style>
