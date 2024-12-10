import { redirect } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";

export function load({ url }: PageLoadEvent) {
	const code = url.searchParams.get("code");

	if (code) {
		redirect(301, `/store/redeem?code=${code}`);
	} else {
		redirect(301, "/store/redeem");
	}
}
