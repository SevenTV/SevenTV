<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import type { PageData } from "./$types";
	import Spinner from "$/components/spinner.svelte";
	import UserEventComponent from "$/components/users/user-event.svelte";
	import type { UserEvent } from "$/gql/graphql";

	let { data }: { data: PageData } = $props();

	async function loadEvents(id: string, page: number) {
		const res = await gqlClient().query(graphql(`
			query UserEvents($id: Id!, $page: Int!) {
				users {
					user(id: $id) {
						events(page: $page, perPage: 100) {
							id
							createdAt
							actor {
								id
								mainConnection {
									platformDisplayName
								}
								highestRoleColor {
									hex
								}
							}
							data {
								__typename
								... on EventUserDataChangeActivePaint {
									newPaint {
										id
										name
										data {
											layers {
												id
												ty {
													__typename
													... on PaintLayerTypeSingleColor {
														color {
															hex
														}
													}
													... on PaintLayerTypeLinearGradient {
														angle
														repeating
														stops {
															at
															color {
																hex
															}
														}
													}
													... on PaintLayerTypeRadialGradient {
														repeating
														stops {
															at
															color {
																hex
															}
														}
														shape
													}
													... on PaintLayerTypeImage {
														images {
															url
															mime
															size
															scale
															width
															height
															frameCount
														}
													}
												}
												opacity
											}
											shadows {
												color {
													hex
												}
												offsetX
												offsetY
												blur
											}
										}
									}
									oldPaint {
										id
										name
										data {
											layers {
												id
												ty {
													__typename
													... on PaintLayerTypeSingleColor {
														color {
															hex
														}
													}
													... on PaintLayerTypeLinearGradient {
														angle
														repeating
														stops {
															at
															color {
																hex
															}
														}
													}
													... on PaintLayerTypeRadialGradient {
														repeating
														stops {
															at
															color {
																hex
															}
														}
														shape
													}
													... on PaintLayerTypeImage {
														images {
															url
															mime
															size
															scale
															width
															height
															frameCount
														}
													}
												}
												opacity
											}
											shadows {
												color {
													hex
												}
												offsetX
												offsetY
												blur
											}
										}
									}
								}
								... on EventUserDataChangeActiveBadge {
									newBadgeId
									oldBadgeId
								}
								... on EventUserDataChangeActiveEmoteSet {
									newEmoteSet {
										id
										name
									}
									oldEmoteSet {
										id
										name
									}
								}
								... on EventUserDataAddConnection {
									addedPlatform
								}
								... on EventUserDataRemoveConnection {
									removedPlatform
								}
							}
						}
					}
				}
			}
		`), { id, page }).toPromise();

		return res.data?.users.user?.events as UserEvent[];
	}

	let page = $state(1);
	let events = $derived(loadEvents(data.id, page));
</script>

<div class="events">
	{#await events}
		<div class="spinner-wrapper">
			<Spinner />
		</div>
	{:then events}
		{#each events as event, index}
			<UserEventComponent {event} />
			{#if index !== events.length - 1}
				<hr />
			{/if}
		{/each}
	{/await}
</div>

<style lang="scss">
	.spinner-wrapper {
		text-align: center;
	}

	.events {
		display: flex;
		flex-direction: column;
	}
</style>
