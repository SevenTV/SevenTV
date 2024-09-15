import { error } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";
import { graphql } from "$/gql";

export async function load({ parent }: PageLoadEvent) {
	const client = (await parent()).client;

	const res = await client
		.query(
			graphql(`
				query EmoteSearch {
					emotes {
						search(query: "test", sort: { sortBy: TOP_ALL_TIME, order: DESCENDING }) {
							id
							defaultName
							ownerId
							flags {
								animated
							}
							images {
								url
								mime
								size
								width
								frameCount
							}
						}
					}
				}
			`),
			{},
		)
		.toPromise();

	if (res.error || !res.data) {
		console.error(res.error);
		throw error(500, {
			message: "Internal server error",
		});
	}

	return {
		results: res.data.emotes.search,
	};
}
