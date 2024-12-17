import { PUBLIC_STRIPE_CUSTOMER_PORTAL } from "$env/static/public";
import { redirect } from "@sveltejs/kit";

export function load() {
	redirect(302, PUBLIC_STRIPE_CUSTOMER_PORTAL);
}
