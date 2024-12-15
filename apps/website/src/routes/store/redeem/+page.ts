import type { PageLoadEvent } from "../../$types";

export function load({ url }: PageLoadEvent) {
	const code = url.searchParams.get("code");

	return {
		code,
	};
}
