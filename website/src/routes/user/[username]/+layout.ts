import type { LayoutLoadEvent } from "./$types";

export async function load({ params }: LayoutLoadEvent) {
	// TODO: fetch user data and return error when not found

	return { username: params.username };
}
