<script lang="ts">
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import { graphql } from "$/gql";
	import type { PageData } from "./$types";
	import type { Emote, EmoteSearchResult } from "$/gql/graphql";
	import type { Client } from "@urql/svelte";

	let { data }: { data: PageData } = $props();

	function load(client: Client, page: number, _perPage: number): Promise<EmoteSearchResult> {
		return client
			.query(
				graphql(`
					query UserActiveEmotes($id: Id!, $page: Int!) {
						users {
							user(id: $id) {
								style {
									activeEmoteSet {
										emotes(page: $page, perPage: 100) {
											items {
												alias
												flags {
													zeroWidth
												}
												emote {
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
											totalCount
											pageCount
										}
									}
								}
							}
						}
					}
				`),
				{
					id: data.user.id,
					page,
				},
			)
			.toPromise()
			.then((res) => {
				if (res.error || !res.data) {
					console.error(res.error);
					throw res.error;
				}

				const emotes = res.data.users.user?.style.activeEmoteSet?.emotes;

				if (!emotes) {
					throw new Error("No emotes found");
				}

				return {
					items: emotes.items
						.filter((item) => item.emote)
						.map((item) => {
							const emote = item.emote!;

							emote.defaultName = item.alias || emote!.defaultName;
							emote.flags.defaultZeroWidth = item.flags.zeroWidth || emote.flags.defaultZeroWidth;

							return emote as Emote;
						}),
					totalCount: emotes.totalCount,
					pageCount: emotes.pageCount,
				};
			});
	}
</script>

<EmoteLoader {load} />
