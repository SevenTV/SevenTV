<script lang="ts">
	import EmotesSent from "./user/emotes-sent.svelte";
	import FavEmotes from "./user/fav-emotes.svelte";
	import EmotesAdded from "./user/emotes-added.svelte";
	import EmotesRemoved from "./user/emotes-removed.svelte";
	import TopCosmetics from "./user/top-cosmetics.svelte";
	import PaintsBadgesCount from "./user/paints-badges-count.svelte";
	import Overview from "./user/overview.svelte";
	import { gqlClient } from "$/lib/gql";
	import type { Paint, Badge } from "$/gql/graphql";
	import { graphql } from "$/gql";
	import { type AnyEvent } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import ThankYou from "./user/thank-you.svelte";
	import Potatbotat from "./user/potatbotat.svelte";

	interface Props {
		potatData: any;
		cosmetics: { paints: Paint[]; badges: Badge[] };
	}
	let { potatData, cosmetics }: Props = $props();

	const channelData = potatData.data[0].channel;
	const userData = potatData.data[0].user;
	const favUserEmotes = { sum_used: userData.sum_used, top_used: userData.top_used };
	const favChannelEmotes = channelData;

	let UserDataPaintsSaved = $state(localStorage.getItem("paintCountsRecap2025"));
	let UserDataBadgesSaved = $state(localStorage.getItem("badgeCountsRecap2025"));
	let UserDataEmoteAddsSaved = $state(localStorage.getItem("totalEmotesAddedRecap2025"));
	let UserDataEmoteRemovesSaved = $state(localStorage.getItem("totalEmotesRemovedRecap2025"));
	let UserDataPaintsSavedTotal = $state(localStorage.getItem("totalPaintsCountRecap2025"));
	let UserDataBadgesSavedTotal = $state(localStorage.getItem("totalBadgesCountRecap2025"));
	let idGeneratedRecap2025 = $state(localStorage.getItem("idGeneratedRecap2025"));

	let isLoadingData = $state(false);

	type ChangeActivePaintEvent = {
		__typename: "UserEvent";
		data: {
			__typename: "EventUserDataChangeActivePaint";
			newPaint?: {
				id: string;
			};
		};
	};

	type ChangeActiveBadgeEvent = {
		__typename: "UserEvent";
		data: {
			__typename: "EventUserDataChangeActiveBadge";
			newBadge?: {
				id: string;
			};
		};
	};

	type RemovedEmoteEvent = {
		__typename: "EmoteSetEvent";
		data: {
			__typename: "EventEmoteSetDataRemoveEmote";
			removedEmote?: {
				defaultName: string;
				id: string;
			};
		};
	};

	type AddEmoteEvent = {
		__typename: "EmoteSetEvent";
		data: {
			__typename: "EventEmoteSetDataAddEmote";
			addedEmote?: {
				defaultName: string;
				id: string;
			};
		};
	};

	function isChangeActivePaintEvent(event: any): event is ChangeActivePaintEvent {
		return (
			event?.__typename === "UserEvent" &&
			event?.data?.__typename === "EventUserDataChangeActivePaint" &&
			typeof event?.data?.newPaint?.id === "string"
		);
	}

	function isChangeActiveBadgeEvent(event: any): event is ChangeActiveBadgeEvent {
		return (
			event?.__typename === "UserEvent" &&
			event?.data?.__typename === "EventUserDataChangeActiveBadge" &&
			typeof event?.data?.newBadge?.id === "string"
		);
	}

	function isAddEmoteEventEvent(event: any): event is AddEmoteEvent {
		return (
			event?.__typename === "EmoteSetEvent" &&
			event?.data?.__typename === "EventEmoteSetDataAddEmote" &&
			typeof event?.data?.addedEmote?.id === "string"
		);
	}

	function isRemovedEmoteEvent(event: any): event is RemovedEmoteEvent {
		return (
			event?.__typename === "EmoteSetEvent" &&
			event?.data?.__typename === "EventEmoteSetDataRemoveEmote" &&
			typeof event?.data?.removedEmote?.id === "string"
		);
	}

	async function loadEvents(id: string, page: number) {
		const res = await gqlClient()
			.query(
				graphql(`
					query UserEvents2($id: Id!, $page: Int!) {
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

	$effect(() => {
		(async () => {
			if (idGeneratedRecap2025 !== $user?.id) {
				// Clear saved data if user ID does not match in case of account switch
				localStorage.removeItem("paintCountsRecap2025");
				localStorage.removeItem("badgeCountsRecap2025");
				localStorage.removeItem("totalEmotesAddedRecap2025");
				localStorage.removeItem("totalEmotesRemovedRecap2025");
				localStorage.removeItem("totalPaintsCountRecap2025");
				localStorage.removeItem("totalBadgesCountRecap2025");
				UserDataPaintsSaved = null;
				UserDataBadgesSaved = null;
				UserDataEmoteAddsSaved = null;
				UserDataEmoteRemovesSaved = null;
				UserDataPaintsSavedTotal = null;
				UserDataBadgesSavedTotal = null;
			}
			if (
				!UserDataPaintsSaved ||
				!UserDataBadgesSaved ||
				!UserDataEmoteAddsSaved ||
				!UserDataEmoteRemovesSaved
			) {
				isLoadingData = true;
				let stvUserId: string | null = null;
				if ($user) {
					stvUserId = $user.id;
				}
				const minDate = new Date("2025-01-01T00:00:00Z").toISOString();
				let page = 1;
				let collected: AnyEvent[] = [];

				while (true) {
					await new Promise((resolve) => setTimeout(resolve, 1000)); // wait 1 second before each request to prevent rate limiting
					const parsed = await loadEvents(stvUserId!, page);
					if (!parsed || parsed.length === 0) break;

					const only2025 = parsed.filter(
						(ev) => ev.createdAt && new Date(ev.createdAt) >= new Date(minDate),
					);
					collected.push(...only2025);

					if (
						page == 10 ||
						parsed.some((ev) => ev.createdAt && new Date(ev.createdAt) < new Date(minDate))
					) {
						break;
					}

					page++;
				}

				const paintCounts: Record<string, { count: number; name?: string }> = {};
				const badgeCounts: Record<string, { count: number; name?: string }> = {};
				let totalEmoteAdded = 0;
				let totalEmoteRemoved = 0;
				let totalPaintsCount = 0;
				let totalBadgesCount = 0;

				for (const event of collected) {
					// Paint
					if (isChangeActivePaintEvent(event)) {
						totalPaintsCount++;
						const paintId = event.data.newPaint?.id;
						const paintName = event.data.newPaint?.name;

						if (paintId) {
							paintCounts[paintId] ??= { count: 0, name: paintName };
							paintCounts[paintId].count++;
						}
					}

					// Badge
					if (isChangeActiveBadgeEvent(event)) {
						totalBadgesCount++;
						const badgeId = event.data.newBadge?.id;
						const badgeName = event.data.newBadge?.name;

						if (badgeId) {
							badgeCounts[badgeId] ??= { count: 0, name: badgeName };
							badgeCounts[badgeId].count++;
						}
					}

					// Emote Added (total only)
					if (isAddEmoteEventEvent(event)) {
						totalEmoteAdded++;
					}

					// Emote Removed (total only)
					if (isRemovedEmoteEvent(event)) {
						totalEmoteRemoved++;
					}
				}
				const paintCountsArray = Object.entries(paintCounts)
					.map(([id, { count, name }]) => ({
						id,
						name,
						count,
					}))
					.sort((a, b) => b.count - a.count)
					.slice(0, 10);

				const badgeCountsArray = Object.entries(badgeCounts)
					.map(([id, { count, name }]) => ({
						id,
						name,
						count,
					}))
					.sort((a, b) => b.count - a.count)
					.slice(0, 10);

				UserDataPaintsSaved = JSON.stringify(paintCountsArray);
				UserDataBadgesSaved = JSON.stringify(badgeCountsArray);
				UserDataEmoteAddsSaved = JSON.stringify(totalEmoteAdded.toLocaleString());
				UserDataEmoteRemovesSaved = JSON.stringify(totalEmoteRemoved.toLocaleString());

				// save in localStorage
				localStorage.setItem("paintCountsRecap2025", JSON.stringify(paintCountsArray));
				localStorage.setItem("badgeCountsRecap2025", JSON.stringify(badgeCountsArray));
				localStorage.setItem(
					"totalEmotesAddedRecap2025",
					JSON.stringify(totalEmoteAdded.toLocaleString()),
				);
				localStorage.setItem(
					"totalEmotesRemovedRecap2025",
					JSON.stringify(totalEmoteRemoved.toLocaleString()),
				);
				localStorage.setItem("totalPaintsCountRecap2025", JSON.stringify(totalPaintsCount));
				localStorage.setItem("totalBadgesCountRecap2025", JSON.stringify(totalBadgesCount));
				UserDataPaintsSavedTotal = JSON.stringify(totalPaintsCount);
				UserDataBadgesSavedTotal = JSON.stringify(totalBadgesCount);
				localStorage.setItem("idGeneratedRecap2025", $user?.id || "");
				idGeneratedRecap2025 = $user?.id || "";
				isLoadingData = false;
			}
		})();
	});
</script>

{#if isLoadingData}
	<section class="hero loading-hero">
		<header class="top">
			<div class="header-content side-by-side">
				<div class="loading-container">
					<div class="spinner"></div>
					<p class="recap subtitle">Loading your 2025 recap data...</p>
					<p class="recap subtitle">It may take a while. Please keep this page open</p>
				</div>
			</div>
		</header>
	</section>
{:else}
	{#if userData && (userData.sum_used != null || userData.top_used != null)}
		<EmotesSent emoteSentCount={userData.sum_used} />
	{/if}
	{#if (favUserEmotes && favUserEmotes.top_used != null) || (favChannelEmotes && favChannelEmotes.top_used != null)}
		<FavEmotes {favUserEmotes} {favChannelEmotes} />
	{/if}
	{#if UserDataEmoteAddsSaved}
		<EmotesAdded
			userEmoteAdds={UserDataEmoteAddsSaved ? JSON.parse(UserDataEmoteAddsSaved) : null}
		/>
	{/if}
	{#if UserDataEmoteRemovesSaved}
		<EmotesRemoved
			userEmoteRemoves={UserDataEmoteRemovesSaved ? JSON.parse(UserDataEmoteRemovesSaved) : null}
		/>
	{/if}
	{#if UserDataPaintsSaved && UserDataBadgesSaved}
		<PaintsBadgesCount
			userDataPaintsCount={UserDataPaintsSavedTotal ? JSON.parse(UserDataPaintsSavedTotal) : null}
			userDataBadgesCount={UserDataBadgesSavedTotal ? JSON.parse(UserDataBadgesSavedTotal) : null}
		/>
	{/if}
	{#if cosmetics && ((UserDataPaintsSaved && UserDataPaintsSaved !== "[]" && UserDataPaintsSaved.length > 0) || (UserDataBadgesSaved && UserDataBadgesSaved !== "[]" && UserDataBadgesSaved.length > 0))}
		<TopCosmetics
			{cosmetics}
			UserDataPaintsSaved={UserDataPaintsSaved ? JSON.parse(UserDataPaintsSaved) : null}
			UserDataBadgesSaved={UserDataBadgesSaved ? JSON.parse(UserDataBadgesSaved) : null}
		/>
	{/if}

	<ThankYou />
	<Overview
		favUserEmotes={favUserEmotes ?? { sum_used: 0, top_used: [] }}
		emoteSentCount={userData.sum_used || 0}
		userEmoteAdds={UserDataEmoteAddsSaved ? JSON.parse(UserDataEmoteAddsSaved) : null}
		userEmoteRemoves={UserDataEmoteRemovesSaved ? JSON.parse(UserDataEmoteRemovesSaved) : null}
		userDataPaintsCount={UserDataPaintsSavedTotal ? JSON.parse(UserDataPaintsSavedTotal) : null}
		userDataBadgesCount={UserDataBadgesSavedTotal ? JSON.parse(UserDataBadgesSavedTotal) : null}
	/>
	<Potatbotat />
{/if}

<style>
	.loading-hero {
		display: flex;
		justify-content: center;
		align-items: center;
		min-height: 40vh;
	}

	.header-content {
		width: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.loading-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		padding: 2rem;
	}

	.spinner {
		width: 48px;
		height: 48px;
		border: 5px solid #eee;
		border-top: 5px solid #7c3aed;
		border-radius: 50%;
		animation: spin 1s linear infinite;
		margin-bottom: 0.5rem;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	.recap.subtitle {
		font-size: 1.2rem;
		text-align: center;
		color: #ffffff;
	}

	@media (max-width: 600px) {
		.loading-hero {
			min-height: 30vh;
			padding: 1rem;
		}
		.loading-container {
			padding: 1rem;
		}
		.spinner {
			width: 36px;
			height: 36px;
		}
		.recap.subtitle {
			font-size: 1rem;
		}
	}
</style>
