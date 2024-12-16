import { graphql } from "$/gql";
import type { EmoteSet } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";
import type { PageLoadEvent } from "./$types";

async function loadSets(fetchF: typeof fetch, id: string) {
	const res = await gqlClient()
		.query(
			graphql(`
				query UserEmoteSets($id: Id!) {
					users {
						user(id: $id) {
							emoteSets {
								id
								name
								capacity
								kind
								emotes(page: 1, perPage: 12) {
									items {
										emote {
											images {
												url
												mime
												size
												scale
												width
												frameCount
											}
										}
									}
									totalCount
								}
							}
						}
					}
				}
			`),
			{ id },
			{ fetch: fetchF },
		)
		.toPromise();

	return res.data?.users.user?.emoteSets as EmoteSet[];
}

export function load({ params, fetch }: PageLoadEvent) {
	return {
		streamed: {
			sets: loadSets(fetch, params.id),
		},
	};
}
