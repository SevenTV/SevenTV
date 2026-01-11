import type { PageLoadEvent } from "./$types";

export function load({ url }: PageLoadEvent) {
	const success = url.searchParams.get("success") === "1";

	return {
		success,
	};
}
