import { error } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";
import { PUBLIC_REST_API_V4 } from "$env/static/public";
import { get } from "svelte/store";
import { sessionToken } from "$/lib/auth";

export async function load({ url, fetch }: PageLoadEvent) {
	const errorName = url.searchParams.get("error");

	if (errorName) {
		const errorDescription = url.searchParams.get("error_description");
		error(500, { message: "Link Error", details: errorDescription });
	}

	const token = get(sessionToken);
	if (!token) {
		error(401, { message: "Unauthorized", details: "You are not logged in" });
	}

	const platform = url.searchParams.get("platform");
	const code = url.searchParams.get("code");
	const returnTo = url.searchParams.get("state");

	if (!code || !platform) {
		error(400, { message: "Invalid URL" });
	}

	if (returnTo) {
		const returnToUrl = new URL(returnTo, url);

		if (returnToUrl.origin !== url.origin) {
			error(400, { message: "Invalid return_to URL" });
		}
	}

	const req = fetch(`${PUBLIC_REST_API_V4}/auth/link/finish`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
		},
		body: JSON.stringify({ platform, code }),
		credentials: "include",
	})
		.then(async (res) => {
			if (!res.ok) {
				throw await res.json();
			}

			return;
		})
		.catch((res) => {
			console.error(res);
			throw res.error;
		});

	return {
		streamed: {
			linkRequest: req,
		},
		returnTo,
	};
}
