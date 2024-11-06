<script lang="ts">
	import EmoteSetPreview from "$/components/emote-set-preview.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { graphql } from "$/gql";
	import type { EmoteSet } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();

	async function loadSets(id: string) {
		const res = await gqlClient().query(
			graphql(`
				query UserEmoteSets($id: Id!) {
					users {
						user(id: $id) {
							ownedEmoteSets {
								id
								name
								capacity
								kind
								emotes(page: 1, perPage: 12) {
									items {
										alias
										flags {
											zeroWidth
										}
										emote {
											defaultName
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
			{ id },
		);

		return res.data?.users.user?.ownedEmoteSets as EmoteSet[];
	}

	let sets = $derived(loadSets(data.user.id));
</script>

{#await sets}
	<div class="spinner-wrapper">
		<Spinner />
	</div>
{:then sets}
	<div class="emote-sets">
		{#each sets as set}
			<EmoteSetPreview data={set} />
		{/each}
	</div>
{/await}

<style lang="scss">
	.spinner-wrapper {
		margin: 0 auto;
	}

	.emote-sets {
		display: grid;
		gap: 1rem;
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
	}
</style>
