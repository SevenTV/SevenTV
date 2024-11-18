import { graphql } from "$/gql";
import { gqlClient } from "./gql";

export async function addEmoteToSet(setId: string, emoteId: string, alias?: string) {
	await gqlClient()
		.mutation(
			graphql(`
				mutation AddEmoteToSet($setId: Id!, $emote: EmoteSetEmoteId!) {
					emoteSet(id: $setId) {
						addEmote(emote: { id: $emote }) {
							id
						}
					}
				}
			`),
			{ setId, emote: { emoteId, alias } },
		)
		.toPromise();
}
