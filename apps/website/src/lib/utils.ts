import { getNumberFormatter } from "svelte-i18n";

export function priceFormat() {
	return getNumberFormatter({
		style: "currency",
		currency: "USD",
	});
}

export function numberFormat() {
	return getNumberFormatter({
		notation: "compact",
		unitDisplay: "narrow",
	});
}
