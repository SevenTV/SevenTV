import { graphql } from "$/gql";
import type { LayoutLoadEvent } from "./$types";
import type { Role, User } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";
import { filterRoles } from "$/lib/utils";

export function load({ fetch, params }: LayoutLoadEvent) {
	const req = gqlClient()
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
		.toPromise()
		.then((res) => {
			if (res.error || !res.data) {
				console.error(res.error);
				throw "Failed to load user";
			}

			if (!res.data.users.user) {
				throw "User not found";
			}

			res.data.users.user.roles = filterRoles(res.data.users.user.roles as Role[]);

			return res.data.users.user as User;
		});

	return {
		id: params.id,
		streamed: {
			userRequest: req,
		},
	};
}
