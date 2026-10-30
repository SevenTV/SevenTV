<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import type { PageData } from "./$types";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import Spinner from "$/components/spinner.svelte";
	import EmoteEventComponent from "$/components/events/emote-event.svelte";
	import type { EmoteEvent } from "$/gql/graphql";
	import { t } from "svelte-i18n";

	let { data }: { data: PageData } = $props();

	async function loadEvents(id: string) {
		try {
			return await loadEventsUnsafe(id);
		} catch (error) {
			console.error("Failed to load emote activity", error);
			throw error;
		}
	}

	async function loadEventsUnsafe(id: string) {
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
										style {
											activeProfilePicture {
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
										highestRoleColor {
											hex
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
													platformAvatarUrl
												}
												style {
													activeProfilePicture {
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
												highestRoleColor {
													hex
												}
											}
											newOwner {
												id
												mainConnection {
													platformDisplayName
													platformAvatarUrl
												}
												style {
													activeProfilePicture {
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
												highestRoleColor {
													hex
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
		{#if events.length === 0}
			<p class="empty">{$t("pages.emote.activity.empty")}</p>
		{:else}
			{#each events as event}
				<EmoteEventComponent {event} />
			{/each}
		{/if}
	{:catch}
		<p class="empty error">{$t("pages.emote.activity.error")}</p>
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

	.empty {
		color: var(--text-light);
		font-size: 0.85rem;
		text-align: center;
		padding: 2rem 0;
	}

	.empty.error {
		color: var(--danger);
	}
</style>
