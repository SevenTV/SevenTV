import { graphql } from "$/gql";
import type { LayoutLoadEvent } from "./$types";
import type { Emote } from "$/gql/graphql";
import { error } from "@sveltejs/kit";

export async function load({ parent, fetch, params }: LayoutLoadEvent) {
	const client = (await parent()).client;

	// TODO: Don't do this in load function because it takes too long
	const req = client.query(graphql(`
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
						highestRoleColor {
							hex
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
							highestRoleColor {
								hex
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
	}).toPromise().then((res) => {
		if (res.error || !res.data) {
			console.error(res.error);
		}

		return res.data?.emotes.emote as Emote;
	});

	return {
		streamed: {
			emote: req,
		}
	};
}
