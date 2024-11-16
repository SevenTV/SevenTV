<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { t } from "svelte-i18n";
	import { user } from "$/lib/auth";
	import PaintComponent from "$/components/paint.svelte";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import Spinner from "$/components/spinner.svelte";
	import type { Paint } from "$/gql/graphql";
	import Expandable from "$/components/expandable.svelte";

	async function queryInventory(id: string) {
		const res = await gqlClient()
			.query(
				graphql(`
					query MyInventory($id: Id!) {
						users {
							user(id: $id) {
								inventory {
									paints {
										from {
											__typename
											... on EntitlementNodeSubscriptionBenefit {
												subscriptionBenefit {
													name
												}
											}
										}
										to {
											paint {
												id
												name
												data {
													layers {
														id
														ty {
															__typename
															... on PaintLayerTypeSingleColor {
																color {
																	hex
																}
															}
															... on PaintLayerTypeLinearGradient {
																angle
																repeating
																stops {
																	at
																	color {
																		hex
																	}
																}
															}
															... on PaintLayerTypeRadialGradient {
																repeating
																stops {
																	at
																	color {
																		hex
																	}
																}
																shape
															}
															... on PaintLayerTypeImage {
																images {
																	url
																	mime
																	size
																	scale
																	width
																	height
																	frameCount
																}
															}
														}
														opacity
													}
													shadows {
														color {
															hex
														}
														offsetX
														offsetY
														blur
													}
												}
											}
										}
									}
								}
							}
						}
					}
				`),
				{ id },
			)
			.toPromise();

		if (res.error || !res.data) {
			throw res.error;
		}

		const inventory = res.data.users.user?.inventory;

		if (!inventory) {
			return undefined;
		}

		const subPaints: { [key: string]: Paint[] } = {};
		const otherPaints = [];

		for (const entitlement of inventory.paints) {
			if (entitlement.from.__typename === "EntitlementNodeSubscriptionBenefit") {
				const benefitName = entitlement.from.subscriptionBenefit.name;

				if (!subPaints[benefitName]) {
					subPaints[benefitName] = [];
				}

				subPaints[benefitName].push(entitlement.to.paint as Paint);
			} else {
				otherPaints.push(entitlement.to.paint as Paint);
			}
		}

		return {
			paints: {
				sub: subPaints,
				other: otherPaints,
			},
		};
	}

	let inventory = $derived($user ? queryInventory($user.id) : undefined);
</script>

<svelte:head>
	<title>Your Cosmetics - {$t("page_titles.suffix")}</title>
</svelte:head>

{#await inventory}
	<Spinner />
{:then inventory}
	{#if inventory}
		<div class="layout">
			<h1>Your Cosmetics</h1>
			<br />
			<hr />

			{#each Object.keys(inventory.paints.sub) as benefitName}
				<h2>{benefitName}</h2>
				<div class="paints">
					{#each inventory.paints.sub[benefitName] as paint}
						<PaintComponent {paint} style="font-size: 1.2rem; font-weight: 700">
							{paint.name.length > 0 ? paint.name : paint.id}
						</PaintComponent>
					{/each}
				</div>
				<br />
				<hr />
			{/each}

			<h2>Other Paints</h2>
			<div class="paints">
				{#each inventory.paints.other as paint}
					<PaintComponent {paint} style="font-size: 1.2rem; font-weight: 700">
						{paint.name.length > 0 ? paint.name : paint.id}
					</PaintComponent>
				{/each}
			</div>
		</div>
	{:else}
		<SignInDialog mode="shown-without-close" />
	{/if}
{/await}

<style lang="scss">
	.layout {
		max-width: 100rem;
		margin: 0 auto;
	}

	.paints {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		gap: 1rem;
	}
</style>
