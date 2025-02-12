import type { PageLoadEvent } from "./$types";

export function load({ url }: PageLoadEvent) {
	const success = url.searchParams.get("success") === "1" || url.searchParams.get("pickems") === "1";
	const redeemSuccess = url.searchParams.get("redeem_success") === "1";

	return {
		success,
		redeemSuccess,
	};
}
