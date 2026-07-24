<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import type { PageData } from "./$types";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import Spinner from "$/components/spinner.svelte";
	import EmoteEventComponent from "$/components/events/emote-event.svelte";
	import type { EmoteEvent } from "$/gql/graphql";

	let { data }: { data: PageData } = $props();

	async function loadEvents(id: string) {
		const res = await gqlClient()
			.query(
				graphql(`
					query EmoteActivityEvents($id: Id!) {
						emotes {
							emote(id: $id) {
								events {
									id
									createdAt
									actor {
										id
										mainConnection {
											platformDisplayName
											platformAvatarUrl
										}
										highestRoleColor {
											hex
										}
										style {
											activeProfilePicture {
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
											activePaint {
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
									}
									data {
										__typename
										... on EventEmoteDataProcess {
											event
										}
										... on EventEmoteDataChangeName {
											oldName
											newName
										}
										... on EventEmoteDataMerge {
											newEmote {
												id
												defaultName
											}
										}
										... on EventEmoteDataChangeOwner {
											oldOwner {
												id
												mainConnection {
													platformDisplayName
												}
												highestRoleColor {
													hex
												}
												style {
													activeProfilePicture {
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
													activePaint {
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
											}
											newOwner {
												id
												mainConnection {
													platformDisplayName
													platformAvatarUrl
												}
												highestRoleColor {
													hex
												}
												style {
													activeProfilePicture {
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
													activePaint {
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
											}
										}
										... on EventEmoteDataChangeTags {
											oldTags
											newTags
										}
										... on EventEmoteDataChangeFlags {
											oldFlags {
												publicListed
												private
												defaultZeroWidth
												approvedPersonal
												deniedPersonal
											}
											newFlags {
												publicListed
												private
												defaultZeroWidth
												approvedPersonal
												deniedPersonal
											}
										}
									}
								}
							}
						}
					}
				`),
				{ id },
			)
			.toPromise();

		if (res.error || !res.data) {
			throw res.error;
		}

		return res.data.emotes.emote?.events as EmoteEvent[];
	}

	let events = $derived(loadEvents(data.id));
</script>

<div class="navigation">
	{#await data.streamed.emote then emote}
		<EmoteTabs id={emote.id} />
	{/await}
</div>
<div class="events">
	{#await events}
		<div class="spinner-wrapper">
			<Spinner />
		</div>
	{:then events}
		{#each events as event, index}
			<EmoteEventComponent {event} />
			{#if index !== events.length - 1}
				<hr />
			{/if}
		{/each}
	{/await}
</div>

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.spinner-wrapper {
		text-align: center;
	}

	.events {
		margin-top: 1.5rem;
	}
</style>
