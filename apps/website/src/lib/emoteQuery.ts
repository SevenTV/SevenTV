import { graphql } from "$/gql";
import type { EmoteSearchResult, Filters, SortBy } from "$/gql/graphql";
import { getContextClient } from "@urql/svelte";

let timeout: NodeJS.Timeout | number | null = null;

export async function queryEmotes(query: string | null, tags: string[], sort: SortBy, filters: Filters | null, page: number | null, perPage: number): Promise<EmoteSearchResult> {
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
						query EmoteSearch($query: String, $tags: [String!]!, $sortBy: SortBy!, $filters: Filters, $page: Int, $perPage: Int!) {
							emotes {
								search(query: $query, tags: { tags: $tags, match: ANY }, sort: { sortBy: $sortBy, order: DESCENDING }, filters: $filters, page: $page, perPage: $perPage) {
									items {
										id
										defaultName
										owner {
											mainConnection {
												platformDisplayName
											}
											highestRoleColor {
												hex
											}
										}
										flags {
											# animated
											# approvedPersonal
											defaultZeroWidth
											# deniedPersonal
											# nsfw
											# private
											publicListed
										}
										images {
											url
											mime
											size
											scale
											width
											frameCount
										}
										ranking(ranking: TRENDING_WEEKLY)
									}
									totalCount
									pageCount
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
						perPage,
					},
				)
				.toPromise();

			if (res.error || !res.data) {
				console.error(res.error);
				throw res.error;
			}

			resolve(res.data.emotes.search as EmoteSearchResult);
		}, 200);
	});
}
