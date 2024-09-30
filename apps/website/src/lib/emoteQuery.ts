import { graphql } from "$/gql";
import type { Emote, Filters, SortBy } from "$/gql/graphql";
import { getContextClient } from "@urql/svelte";

let timeout: NodeJS.Timeout | number | null = null;

export async function queryEmotes(query: string | null, tags: string[], sort: SortBy, filters: Filters | null, page: number | null, limit: number): Promise<Emote[]> {
	if (timeout) {
		clearTimeout(timeout);
	}

	const client = getContextClient();

	// Small timeout to prevent spamming requests when user is typing

	return new Promise((resolve) => {
		timeout = setTimeout(async () => {
			const res = await client
				.query(
					graphql(`
						query EmoteSearch($query: String, $tags: [String!]!, $sortBy: SortBy!, $filters: Filters, $page: Int, $limit: Int!) {
							emotes {
								search(query: $query, tags: { tags: $tags, match: ANY }, sort: { sortBy: $sortBy, order: DESCENDING }, filters: $filters, page: $page, limit: $limit) {
									id
									defaultName
									owner {
										mainConnection {
											platformDisplayName
										}
									}
									flags {
										animated
									}
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
						}
					`),
					{
						query,
						tags,
						sortBy: sort,
						filters,
						page,
						limit: limit,
					},
				)
				.toPromise();

			if (res.error || !res.data) {
				console.error(res.error);
				throw res.error;
			}

			resolve(res.data.emotes.search as Emote[]);
		}, 200);
	});
}
