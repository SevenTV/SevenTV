import { graphql } from "$/gql";
import type { EmoteSet } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";
import { error } from "@sveltejs/kit";
import type { PageLoadEvent } from "./$types";

export async function load({ params, fetch }: PageLoadEvent) {
	const res = await gqlClient()
		.query(
			graphql(`
				query OneSet($id: Id!) {
					emoteSets {
						emoteSet(id: $id) {
							id
							name
							capacity
							kind
							tags
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
			`),
			{ id: params.id },
			{ fetch },
		)
		.toPromise();

	if (res.error || !res.data) {
		error(500, "Failed to load emote set");
	}

	if (!res.data.emoteSets.emoteSet) {
		error(404, "Emote Set not found");
	}

	return res.data.emoteSets.emoteSet as EmoteSet;
}
