import { graphql } from "$/gql";
import { gqlClient } from "./gql";

export async function addEmoteToSet(setId: string, emoteId: string, alias?: string) {
	await gqlClient()
		.mutation(
			graphql(`
				mutation AddEmoteToSet($setId: Id!, $emote: EmoteSetEmoteId!) {
					emoteSets {
						emoteSet(id: $setId) {
							addEmote(emote: { id: $emote }) {
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
