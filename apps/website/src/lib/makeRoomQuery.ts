import { graphql } from "$/gql";
import type { EmoteScores, Image } from "$/gql/graphql";
import { gqlClient } from "./gql";

export type MakeRoomEmote = {
	id: string;
	alias: string;
	emote: {
		id: string;
		defaultName: string;
		images: Image[];
		scores: Pick<EmoteScores, "topDaily" | "topWeekly" | "topMonthly" | "topAllTime">;
	};
};

export type TimeWindow = "topDaily" | "topWeekly" | "topMonthly" | "topAllTime";

export async function fetchEmoteSetEmotesWithScores(setId: string): Promise<MakeRoomEmote[]> {
	const allItems: MakeRoomEmote[] = [];
	let page = 1;
	const perPage = 300;

	while (true) {
		const res = await gqlClient()
			.query(
				graphql(`
					query MakeRoomEmotes($id: Id!, $page: Int!, $perPage: Int!) {
						emoteSets {
							emoteSet(id: $id) {
								emotes(page: $page, perPage: $perPage) {
									items {
										id
										alias
										emote {
											id
											defaultName
											images {
												url
												mime
												size
												scale
												width
												frameCount
											}
											scores {
												topDaily
												topWeekly
												topMonthly
												topAllTime
											}
										}
									}
									pageCount
								}
							}
						}
					}
				`),
				{ id: setId, page, perPage },
			)
			.toPromise();

		const emotes = res.data?.emoteSets?.emoteSet?.emotes;
		if (!emotes) break;

		allItems.push(...(emotes.items as MakeRoomEmote[]));

		if (page >= emotes.pageCount) break;
		page++;
	}

	return allItems;
}
