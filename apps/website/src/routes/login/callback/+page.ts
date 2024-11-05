import { error, redirect } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";
import { PUBLIC_REST_API_V4 } from "$env/static/public";
import { sessionToken } from "$/lib/auth";

export async function load({ url, fetch }: PageLoadEvent) {
	const errorName = url.searchParams.get("error");

	if (errorName) {
		const errorDescription = url.searchParams.get("error_description");
		error(500, { message: "Sign-in Error", details: errorDescription });
	}

	const platform = url.searchParams.get("platform");
	const code = url.searchParams.get("code");
	const state = url.searchParams.get("state");

	if (!code || !state || !platform) {
		error(400, { message: "Invalid URL" });
	}

	const data = await fetch(`${PUBLIC_REST_API_V4}/auth/login/finish`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ platform, code, state }),
		credentials: "include",
	}).then((res) => res.json());

	if (data.error || !data.token) {
		console.error(data);
		error(500, { message: "Sign-in Error", details: data.error });
	}

	sessionToken.set(data.token);

	let payload = null;
	const splitToken = data.token.split(".");
	if (splitToken[1]) {
		try {
			payload = JSON.parse(atob(splitToken[1]));
		} catch (e) {
			console.error(e);
		}
	}

	if (payload.sub) {
		redirect(303, `/users/${payload.sub}`);
	}

	redirect(303, "/");
}
