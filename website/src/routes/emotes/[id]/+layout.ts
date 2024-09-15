import { graphql } from "$/gql";
import { error } from "@sveltejs/kit";
import type { LayoutLoadEvent } from "./$types";

export async function load({ parent, params }: LayoutLoadEvent) {
	const client = (await parent()).client;

	const res = await client.query(graphql(`
		query OneEmote($id: Id!) {
			emotes {
				emote(id: $id) {
					id
					defaultName
					owner {
						mainConnection {
							platformDisplayName
						}
					}
					tags
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
	`), {
		id: params.id,
	}).toPromise();

	if (res.error || !res.data) {
		console.error(res.error);
		throw error(500, "Failed to load emote");
	}

	if (!res.data.emotes.emote) {
		throw error(404, "Emote not found");
	}

	return {
		emote: res.data.emotes.emote,
	};
}
