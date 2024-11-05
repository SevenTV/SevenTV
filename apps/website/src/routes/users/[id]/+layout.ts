import { graphql } from "$/gql";
import { error } from "@sveltejs/kit";
import type { LayoutLoadEvent } from "./$types";
import type { User } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";

export async function load({ fetch, params }: LayoutLoadEvent) {
	// TODO: Don't do this in load function because it takes too long
	const res = await gqlClient()
		.query(
			graphql(`
				query OneUser($id: Id!) {
					users {
						user(id: $id) {
							id
							connections {
								platform
								platformUsername
								platformDisplayName
							}
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
							roles {
								name
								color {
									hex
								}
							}
							editors {
								editor {
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
								state
							}
						}
					}
				}
			`),
			{
				id: params.id,
			},
			{
				fetch,
			},
		)
		.toPromise();

	if (res.error || !res.data) {
		console.error(res.error);
		error(500, "Failed to load user");
	}

	if (!res.data.users.user) {
		error(404, "User not found");
	}

	return {
		user: res.data.users.user as User,
	};
}
