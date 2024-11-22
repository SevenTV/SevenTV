import { graphql } from "$/gql";
import type { EmoteSet } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";
import type { PageLoadEvent } from "./$types";

async function loadSets(id: string) {
	const res = await gqlClient().query(
		graphql(`
			query UserEmoteSets($id: Id!) {
				users {
					user(id: $id) {
						ownedEmoteSets {
							id
							name
							capacity
							kind
							emotes(page: 1, perPage: 12) {
								items {
									alias
									flags {
										zeroWidth
									}
									emote {
										defaultName
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
	);

	return res.data?.users.user?.ownedEmoteSets as EmoteSet[];
}

export function load({ params }: PageLoadEvent) {
	return {
		streamed: {
			sets: loadSets(params.id),
		},
	};
}
