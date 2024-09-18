import { graphql } from "$/gql";
import type { Emote, SortBy } from "$/gql/graphql";
import { getContextClient } from "@urql/svelte";

export async function queryEmotes(query: string | null, limit: number, page: number | null, sort: SortBy): Promise<Emote[]> {
    const res = await getContextClient()
        .query(
            graphql(`
                query EmoteSearch($query: String, $limit: Int!, $page: Int, $sortBy: SortBy!) {
                    emotes {
                        search(query: $query, sort: { sortBy: $sortBy, order: DESCENDING }, limit: $limit, page: $page) {
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
                                width
                                frameCount
                            }
                        }
                    }
                }
            `),
            {
                query,
                limit: limit,
                page,
                sortBy: sort,
            },
        )
        .toPromise();

    if (res.error || !res.data) {
        console.error(res.error);
        throw res.error;
    }

    return res.data.emotes.search as Emote[];
}
