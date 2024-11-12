import type { PageLoadEvent } from "./$types";
import { redirect } from "@sveltejs/kit";

export function load({ params }: PageLoadEvent) {
    redirect(301, `/users/${params.id}`);
}
