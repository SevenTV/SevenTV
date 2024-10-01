import { graphql } from "$/gql";
import { error } from "@sveltejs/kit";
import type { LayoutLoadEvent } from "./$types";
import type { Emote } from "$/gql/graphql";

export async function load({ parent, fetch, params }: LayoutLoadEvent) {
	const client = (await parent()).client;

	const res = await client.query(graphql(`
		query OneEmote($id: Id!) {
			emotes {
				emote(id: $id) {
					id
					defaultName
					owner {
						id
						mainConnection {
							platformDisplayName
							platformAvatarUrl
						}
						style {
							activeProfilePicture {
								images {
									url
									mime
									size
									width
									height
									scale
									frameCount
								}
							}
						}
					}
					tags
					flags {
						defaultZeroWidth
						publicListed
					}
					attribution {
						user {
							mainConnection {
								platformDisplayName
								platformAvatarUrl
							}
							style {
								activeProfilePicture {
									images {
										url
										mime
										size
										width
										height
										scale
										frameCount
									}
								}
							}
						}
					}
					images {
						url
						mime
						size
						width
						height
						scale
						frameCount
					}
				}
			}
		}
	`), {
		id: params.id,
	}, {
		fetch,
	}).toPromise();

	if (res.error || !res.data) {
		console.error(res.error);
		throw error(500, "Failed to load emote");
	}

	if (!res.data.emotes.emote) {
		throw error(404, "Emote not found");
	}

	return {
		emote: res.data.emotes.emote as Emote,
	};
}
