import { error } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";
import { PUBLIC_REST_API_V4 } from "$env/static/public";

export async function load({ url, fetch }: PageLoadEvent) {
	const errorName = url.searchParams.get("error");

	if (errorName) {
		const errorDescription = url.searchParams.get("error_description");
		error(500, { message: "Sign-in Error", details: errorDescription });
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

	const req = fetch(`${PUBLIC_REST_API_V4}/auth/login/finish`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ platform, code }),
		credentials: "include",
	})
		.then((res) => res.json())
		.then((res) => {
			if (res.error || !res.token || typeof res.token !== "string") {
				console.error(res);
				throw res.error;
			}

			return res.token as string;
		});

	return {
		streamed: {
			loginRequest: req,
		},
		returnTo,
	};
}
