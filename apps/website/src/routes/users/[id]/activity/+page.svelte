<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import type { PageData } from "./$types";
	import Spinner from "$/components/spinner.svelte";
	import UserEventComponent from "$/components/events/user-event.svelte";
	import EmoteSetEventComponent from "$/components/events/emote-set-event.svelte";
	import type { AnyEvent } from "$/gql/graphql";

	let { data }: { data: PageData } = $props();

	async function loadEvents(id: string, page: number) {
		const res = await gqlClient()
			.query(
				graphql(`
					query UserEvents($id: Id!, $page: Int!) {
						users {
							user(id: $id) {
								relatedEvents(page: $page, perPage: 100) {
									__typename
									... on UserEvent {
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
												newBadge {
													id
													name
													description
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
												oldBadge {
													id
													name
													description
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
									... on EmoteSetEvent {
										id
										createdAt
										target {
											id
											name
										}
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
											... on EventEmoteSetDataChangeName {
												oldName
												newName
											}
											... on EventEmoteSetDataChangeTags {
												oldTags
												newTags
											}
											... on EventEmoteSetDataChangeCapacity {
												oldCapacity
												newCapacity
											}
											... on EventEmoteSetDataAddEmote {
												alias
												addedEmote {
													id
													defaultName
													images {
														url
														mime
														size
														width
														height
														scale
														frameCount
													}
												}
											}
											... on EventEmoteSetDataRemoveEmote {
												removedEmote {
													id
													defaultName
													images {
														url
														mime
														size
														width
														height
														scale
														frameCount
													}
												}
											}
											... on EventEmoteSetDataRenameEmote {
												oldAlias
												newAlias
												renamedEmote {
													id
													defaultName
													images {
														url
														mime
														size
														width
														height
														scale
														frameCount
													}
												}
											}
										}
									}
								}
							}
						}
					}
				`),
				{ id, page },
			)
			.toPromise();

		return res.data?.users.user?.relatedEvents as AnyEvent[];
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
			{#if event.__typename === "UserEvent"}
				<UserEventComponent {event} />
			{:else if event.__typename === "EmoteSetEvent"}
				<EmoteSetEventComponent {event} />
			{/if}
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

		overflow: auto;
		scrollbar-gutter: stable;
	}
</style>
