<script lang="ts">
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import { graphql } from "$/gql";
	import type { PageData } from "./$types";
	import type { Emote, EmoteSearchResult } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";

	let { data }: { data: PageData } = $props();

	function load(_page: number, _perPage: number): Promise<EmoteSearchResult> {
		return gqlClient()
			.query(
				graphql(`
					query UserUploadedEmotes($id: Id!) {
						users {
							user(id: $id) {
								ownedEmotes {
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
							}
						}
					}
				`),
				{
					id: data.id,
				},
			)
			.toPromise()
			.then((res) => {
				if (res.error || !res.data) {
					console.error(res.error);
					throw res.error;
				}

				const emotes = res.data.users.user?.ownedEmotes;

				if (!emotes) {
					throw new Error("No emotes found");
				}

				return {
					items: emotes as Emote[],
					totalCount: emotes.length,
					pageCount: 1,
				};
			});
	}
</script>

<EmoteLoader {load} />
