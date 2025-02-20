
import { user } from "$/lib/auth";
import { gqlClient } from "$/lib/gql";
import { graphql } from "$/gql";
import { get } from "svelte/store";
import { signInDialogMode, signInDialogPayload } from "./layout";
import type { User } from "$/gql/graphql";
export async function purchasePickems(variantId?: string, waitForUser = false) {
	let $user: User | null | undefined;
	if (!waitForUser) {
		$user = get(user);
	} else {
		$user = await new Promise<User | undefined>((res) => {
			const unsub = user.subscribe((u) => {
				if (!u) return;
				unsub()
				res(u)
			})
		});
	}

	console.log($user, waitForUser);
	if (!$user) {
		// needs to be not undefined as to be JSON.stringable
		signInDialogPayload.set({ pickems: variantId });
		signInDialogMode.set("shown")
		return new Promise<void>((res) => {
			const unsub = signInDialogMode.subscribe((v) => {
				if (v !== "hidden") return;
				unsub()
				res()
			}, res)
		})
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
			{ userId: $user.id, pickemsId, subscriptionPriceId: variantId },
		)
		.toPromise();

	if (res.data) {
		window.location.href = res.data.billing.getPickems.checkoutUrl;
	}
}
