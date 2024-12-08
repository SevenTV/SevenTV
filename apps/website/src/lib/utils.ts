import { SubscriptionProductKind, type Role, type SubscriptionProductVariant } from "$/gql/graphql";
import { getNumberFormatter } from "svelte-i18n";

export function priceFormat(currency: string) {
	return getNumberFormatter({
		style: "currency",
		currency,
	});
}

export function numberFormat() {
	return getNumberFormatter({
		notation: "compact",
		unitDisplay: "narrow",
	});
}

export function isMobileLayout(): boolean {
	return window.matchMedia("screen and (max-width: 960px)").matches;
}

export function filterRoles(roles: Role[]) {
	return roles.filter((r) => r.name !== "Default").reverse();
}

export function variantName(variant: SubscriptionProductVariant) {
	let name;

	switch (variant.kind) {
		case SubscriptionProductKind.Monthly:
			name = "Monthly";
			break;
		case SubscriptionProductKind.Yearly:
			name = "Yearly";
			break;
		default:
			name = variant.kind;
	}

	const price = priceFormat(variant.price.currency).format(variant.price.amount / 100);

	return `${name} – ${price}`;
}

export function compareTags(a: string[], b: string[]) {
	if (a.length !== b.length) return false;

	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false;
	}

	return true;
}
