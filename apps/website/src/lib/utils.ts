import {
	Platform,
	SubscriptionProductKind,
	type Role,
	type SubscriptionProductVariant,
} from "$/gql/graphql";
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
	switch (variant.kind) {
		case SubscriptionProductKind.Monthly:
			return "Monthly";
		case SubscriptionProductKind.Yearly:
			return "Yearly";
		default:
			return variant.kind;
	}
}

export function variantPrice(variant: SubscriptionProductVariant) {
	return priceFormat(variant.price.currency).format(variant.price.amount / 100);
}

export function variantUnit(variant: SubscriptionProductVariant) {
	switch (variant.kind) {
		case SubscriptionProductKind.Monthly:
			return "month";
		case SubscriptionProductKind.Yearly:
			return "year";
	}
}

export function compareTags(a: string[], b: string[]) {
	if (a.length !== b.length) return false;

	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false;
	}

	return true;
}

export function platformToValue(platform: Platform, platformId: string) {
	return `${platform}:${platformId}`;
}

export function valueToPlatform(value: string) {
	const idx = value.indexOf(":");
	return { platform: value.slice(0, idx) as Platform, platformId: value.slice(idx + 1) };
}
