import { user } from "$/lib/auth";
import { gqlClient } from "$/lib/gql";
import { graphql } from "$/gql";
import { get } from "svelte/store";
import { signInDialogMode, signInDialogPayload } from "./layout";
import type { GetPickemsCosmeticsQuery, Paint, User } from "$/gql/graphql";
export async function purchasePickems(
	variantId?: string,
	recipientId?: string,
	waitForUser = false,
) {
	let $user: User | null | undefined;
	if (!waitForUser) {
		$user = get(user);
	} else {
		$user = await new Promise<User | undefined>((res) => {
			const unsub = user.subscribe((u) => {
				if (!u) return;
				unsub();
				res(u);
			});
		});
	}

	if (!$user) {
		// needs to be not undefined as to be JSON.stringable
		signInDialogPayload.set({ pickems: `${variantId},${recipientId}` });
		signInDialogMode.set("shown");
		return new Promise<void>((res) => {
			const unsub = signInDialogMode.subscribe((v) => {
				if (v !== "hidden") return;
				unsub();
				res();
			}, res);
		});
	}

	// FIX update this for live
	const pickemsId = "01JK6K2GCE06A9F6FSBVZA2KQA";

	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation PurchasePickems(
					$userId: Id!
					$pickemsId: Id!
					$subscriptionPriceId: StripeProductId
				) {
					billing(userId: $userId) {
						getPickems(pickemsId: $pickemsId, subscriptionPriceId: $subscriptionPriceId) {
							checkoutUrl
						}
					}
				}
			`),
			{ userId: recipientId ?? $user.id, pickemsId, subscriptionPriceId: variantId },
		)
		.toPromise();

	if (res.data) {
		window.location.href = res.data.billing.getPickems.checkoutUrl;
	}
}

const badgeIds = new Set([
	"01JJQEA3655687JWHG7P9CV3W8",
	"01JJQECTG04J5J6QE1BCATE6JN",
	"01JJQEDT21JXF1JM4F1P805VTK",
	"01JJQEENE6CJ6KR70CBCF39ACN",
]);

const paintIds = new Set(["01JHXJH9C9MHN9FJMPNQB4YZ4N", "01JHXJCX6WPJR5ETEYRDP6WVYH"]);

export async function queryPickemsCosmetics() {
	const res = await gqlClient()
		.query(
			graphql(`
				query GetPickemsCosmetics {
					badges {
						badges {
							__typename
							id
							name
							description
							tags
							images {
								__typename
								url
								mime
								size
								scale
								width
								height
								frameCount
							}
							createdById
							updatedAt
							searchUpdatedAt
						}
					}
					paints {
						paints {
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
			`),
			{},
		)
		.toPromise();

	const data = res.data as GetPickemsCosmeticsQuery;

	const badges = data?.badges.badges.filter((b) => badgeIds.has(b.id)) ?? [];
	const paints = (data?.paints.paints.filter((p) => paintIds.has(p.id)) ?? []) as Paint[];
	return { badges, paints };
}
