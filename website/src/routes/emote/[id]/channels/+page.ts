import type { PageLoadEvent } from "./$types";
import { redirect } from "@sveltejs/kit";

export function load({ params }: PageLoadEvent) {
	redirect(301, `/emote/${params.id}`);
}
