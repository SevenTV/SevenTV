import { graphql } from "$/gql";
import type { EmoteSet } from "$/gql/graphql";
import { gqlClient } from "./gql";

export async function createSet(ownerId: string, name: string, tags: string[]) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation CreateEmoteSet($name: String!, $tags: [String!]!, $ownerId: Id!) {
					emoteSets {
						create(name: $name, tags: $tags, ownerId: $ownerId) {
							id
						}
					}
				}
			`),
			{ name, tags, ownerId },
		)
		.toPromise();

	if (!res.data) {
		return undefined;
	}

	return res.data.emoteSets.create as EmoteSet;
}

export async function addEmoteToSet(setId: string, emoteId: string, alias?: string) {
	await gqlClient()
		.mutation(
			graphql(`
				mutation AddEmoteToSet($setId: Id!, $emote: EmoteSetEmoteId!) {
					emoteSets {
						emoteSet(id: $setId) {
							addEmote(id: $emote) {
								id
							}
						}
					}
				}
			`),
			{ setId, emote: { emoteId, alias } },
		)
		.toPromise();
}

export async function removeEmoteFromSet(setId: string, emoteId: string, alias?: string) {
	await gqlClient()
		.mutation(
			graphql(`
				mutation RemoveEmoteFromSet($setId: Id!, $emote: EmoteSetEmoteId!) {
					emoteSets {
						emoteSet(id: $setId) {
							removeEmote(id: $emote) {
								id
							}
						}
					}
				}
			`),
			{ setId, emote: { emoteId, alias } },
		)
		.toPromise();
}

export async function renameEmoteInSet(
	setId: string,
	emoteId: string,
	newAlias: string,
	oldAlias?: string,
) {
	await gqlClient()
		.mutation(
			graphql(`
				mutation RenameEmoteInSet($setId: Id!, $emote: EmoteSetEmoteId!, $alias: String!) {
					emoteSets {
						emoteSet(id: $setId) {
							updateEmoteAlias(id: $emote, alias: $alias) {
								id
							}
						}
					}
				}
			`),
			{ setId, emote: { emoteId, alias: oldAlias }, alias: newAlias },
		)
		.toPromise();
}

export async function updateName(id: string, name: string) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation UpdateEmoteSetName($id: Id!, $name: String!) {
					emoteSets {
						emoteSet(id: $id) {
							name(name: $name) {
								id
								name
								capacity
								kind
								tags
								owner {
									id
									editors {
										editorId
										state
										permissions {
											emoteSet {
												manage
											}
											user {
												manageProfile
											}
										}
									}
									permissions {
										emoteSetCapacity
										personalEmoteSetCapacity
									}
								}
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
			{ id, name },
		)
		.toPromise();

	if (!res.data) {
		return undefined;
	}

	return res.data.emoteSets.emoteSet.name as EmoteSet;
}

export async function updateCapacity(id: string, capacity: number) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation UpdateEmoteSetCapacity($id: Id!, $capacity: Int!) {
					emoteSets {
						emoteSet(id: $id) {
							capacity(capacity: $capacity) {
								id
								name
								capacity
								kind
								tags
								owner {
									id
									editors {
										editorId
										state
										permissions {
											emoteSet {
												manage
											}
											user {
												manageProfile
											}
										}
									}
									permissions {
										emoteSetCapacity
										personalEmoteSetCapacity
									}
								}
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
			{ id, capacity },
		)
		.toPromise();

	if (!res.data) {
		return undefined;
	}

	return res.data.emoteSets.emoteSet.capacity as EmoteSet;
}

export async function updateTags(id: string, tags: string[]) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation UpdateEmoteSetTags($id: Id!, $tags: [String!]!) {
					emoteSets {
						emoteSet(id: $id) {
							tags(tags: $tags) {
								id
								name
								capacity
								kind
								tags
								owner {
									id
									editors {
										editorId
										state
										permissions {
											emoteSet {
												manage
											}
											user {
												manageProfile
											}
										}
									}
									permissions {
										emoteSetCapacity
										personalEmoteSetCapacity
									}
								}
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
			{ id, tags },
		)
		.toPromise();

	if (!res.data) {
		return undefined;
	}

	return res.data.emoteSets.emoteSet.tags as EmoteSet;
}

export async function deleteSet(id: string) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation DeleteSet($id: Id!) {
					emoteSets {
						emoteSet(id: $id) {
							delete
						}
					}
				}
			`),
			{ id },
		)
		.toPromise();

	return res.data?.emoteSets.emoteSet.delete;
}
