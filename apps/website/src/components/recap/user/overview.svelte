<script lang="ts">
	import SevenTVLogo from "../../icons/logo.svelte";
	import UserProfilePicture from "$/components/user-profile-picture.svelte";
	import UserName from "$/components/user-name.svelte";
	import { user } from "$/lib/auth";
	import type { Badge, Paint } from "$/gql/graphql";
	import BadgeComponent from "$/components/badge.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";

	async function queryInventory(id: string) {
		const res = await gqlClient()
			.query(
				graphql(`
					query UserInventory($id: Id!) {
						users {
							user(id: $id) {
								inventory(includeInaccessible: true) {
									badges {
										accessible
										to {
											badge {
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
									paints {
										accessible
										from {
											__typename
											... on EntitlementNodeRole {
												role {
													id
													name
												}
											}
											... on EntitlementNodeSubscriptionBenefit {
												subscriptionBenefit {
													id
													name
												}
											}
											... on EntitlementNodeSpecialEvent {
												specialEvent {
													id
													name
												}
											}
										}
										to {
											paint {
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

		const inventory = res.data.users.user?.inventory;

		if (!res.data.users.user || !inventory) {
			return undefined;
		}

		const badges = inventory.badges
			.filter((b) => b.to.badge)
			.reduce(
				(map, b) => {
					map[b.to.badge!.id] = {
						...(b.to.badge as Badge),
						accessible: b.accessible,
					};
					return map;
				},
				{} as { [key: string]: Badge & { accessible?: boolean } },
			);

		return {
			badges,
		};
	}

	type potatEmoteFormat = {
		id: string;
		name: string;
		alias: string;
		provider: string;
		use_count: number;
		urls: { url: string; mime: string; size: number; scale: number }[];
	};

	interface Props {
		emoteSentCount: number;
		userEmoteAdds: string;
		userEmoteRemoves: string;
		userDataBadgesCount: string;
		userDataPaintsCount: string;
		favUserEmotes: { sum_used: number; top_used: potatEmoteFormat[] };
	}
	const {
		emoteSentCount,
		userEmoteAdds,
		userEmoteRemoves,
		userDataBadgesCount,
		userDataPaintsCount,
		favUserEmotes,
	}: Props = $props();

	let recapEl: HTMLElement;

	const leaderboardUser = favUserEmotes.top_used?.map((item, index) => ({
		rank: index + 1,
		name: item.name,
		alias: item.alias,
		id: item.id,
		image: `https://cdn.7tv.app/emote/${item.id}/4x.webp`,
		score: item.use_count,
	}));
</script>

<div class="page">
	<div class="recap">
		<div bind:this={recapEl}>
			<div class="header">
				<div class="avatar">
					{#if $user}
						<UserProfilePicture
							user={$user}
							size={5 * 18}
							style="display: flex;
			align-items: center;"
						/>
					{/if}
				</div>
				<div class="header-middle">
					<div class="badges-grid">
						<h1
							style="width: 100%; text-align: center; font-size: 0.7rem; position: static; top: 0; left: 0;"
						>
							BADGES:
						</h1>
						{#if $user}
							{#await queryInventory($user.id) then data}
								{#if data && data.badges}
									{#each Object.values(data.badges) as badge}
										<BadgeComponent
											{badge}
											enableDialog
											style="margin-top: 0.5rem;"
											size={Object.keys(data.badges)?.length < 20
												? 24
												: Object.keys(data.badges)?.length < 30
													? 18
													: Object.keys(data.badges)?.length < 40
														? 16
														: 14}
										/>
									{/each}
								{/if}
							{/await}
						{/if}
					</div>
				</div>
				<div class="header-right">
					<div class="side-by-side">
						<SevenTVLogo size={90} style="color: white; " />
						<p class="long-line">|</p>
						<div class="year-div">
							<h1 class="year">20</h1>
							<h1 class="year">25</h1>
						</div>
					</div>
					<h1 style="color:#ff3f7f; font-weight: 200">RECAP</h1>
				</div>
				<div class="username" title={ $user ? $user.mainConnection?.platformDisplayName : '' }>
					{#if $user}<UserName
							user={$user}
							style="max-width: 35rem; text-overflow: ellipsis;
                                    overflow: hidden;
                                    white-space: nowrap;     display: inline-block;"
						/>{/if}
				</div>
			</div>
			<div class="main-stat">
				<span class="label">EMOTES SENT</span>
				<span class="value">{emoteSentCount.toLocaleString()}</span>
			</div>

			<div class="content">
				<div class="top-emotes">
					<h3><span>Top</span> Emotes</h3>
					{#if leaderboardUser && leaderboardUser != null}
						{#each leaderboardUser.slice(0, 5) as emote, i}
							<div class="emote-row">
								<span class="rank">{i + 1}.</span>
								<div class="emote-box">
									<a
										style="color: white;"
										href={`https://7tv.app/emotes/${emote.id}`}
										target="_blank"
										rel="noopener noreferrer"
										><img
											src={emote.image}
											alt="{emote.alias !== '' ? emote.alias : emote.name}'s avatar"
											style="height: 2rem; max-width :3rem; border-radius: 4px;"
										/></a
									>
								</div>
								<span class="emote-name"
									><a
										style="color: {i % 2 === 1 ? '#ffffff' : '#000000'};
                                    text-overflow: ellipsis;
                                    overflow: hidden;
                                    white-space: nowrap;
                                    max-width: 11.2rem;
                                    display: inline-block;
                                    "
										href={`https://7tv.app/emotes/${emote.id}`}
										target="_blank"
										rel="noopener noreferrer">{emote.alias !== "" ? emote.alias : emote.name}</a
									></span
								>
							</div>
						{/each}
					{/if}
				</div>

				<div class="side-stats">
					<div class="stat">
						<span class="title">Emotes added</span>
						<span class="number pink">{userEmoteAdds}</span>
					</div>

					<div class="stat">
						<span class="title">Emotes removed</span>
						<span class="number pink">{userEmoteRemoves}</span>
					</div>

					<div class="stat">
						<span class="title">Paints changes</span>
						<span class="number pink">{userDataPaintsCount}</span>
					</div>

					<div class="stat">
						<span class="title">Badge changes</span>
						<span class="number pink">{userDataBadgesCount}</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style lang="scss">
	.badges-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 0.3rem;
		justify-content: center;
		align-items: center;
		max-width: 37rem;
	}
	.capture * {
		backdrop-filter: none !important;
		filter: none !important;
		animation: none !important;
	}
	.side-by-side {
		display: flex;
		align-items: center;
		gap: 1rem;
		justify-content: flex-end;
	}
	.long-line {
		font-size: 1rem;
		color: #ffffff;
		transform: scaleY(4.5);
	}
	.download-btn-container {
		display: flex;
		justify-content: center;
		margin: 2.5rem 0 1.5rem 0;
	}

	.download-btn:hover {
		transform: scale(1.05);
		box-shadow: 0 1rem 2.5rem rgba(255, 63, 127, 0.18);
	}
	.page {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100vh;
		background: radial-gradient(circle at top, #141420 0%, #0b0b0f 60%);
	}

	.recap {
		margin-top: 5rem;
		width: 40rem;
		background: #0f0f12;
		border: 0.2rem solid #ffffff;
		font-family: "Inter", system-ui, sans-serif;
		color: #ffffff;
		box-shadow:
			0 2rem 5rem rgba(0, 0, 0, 0.6),
			inset 0 0 0 0.1rem rgba(255, 255, 255, 0.05);
		font-family: "BBH Bartleby", sans-serif;

		.header {
			display: grid;
			grid-template-columns: auto 1fr auto;
			align-items: center;
			padding: 1rem;
			border-bottom: 0.2rem solid #ffffff;

			&-right {
				text-align: right;
				line-height: 1;

				.logo {
					font-size: 2.2rem;
					font-weight: 400;
				}

				.year {
					font-size: 1.8rem;
					font-weight: 400;
					color: #7b61ff;
				}

				.recap-text {
					font-size: 1rem;
					font-weight: 400;
					letter-spacing: 0.15rem;
					color: #ff3f7f;
				}
			}

			&-middle {
				text-align: center;
				line-height: 1;
			}

			.username {
				grid-column: 1 / -1;
				margin-top: 0.75rem;
				font-size: 1.9rem;
				font-weight: 400;
			}
		}

		.main-stat {
			background: #ffffff;
			color: #000000;
			text-align: center;
			padding: 1.75rem 1rem;

			.label {
				display: block;
				font-size: 2rem;
				font-weight: 300;
				letter-spacing: 0.15rem;
			}

			.value {
				display: block;
				margin-top: 0.5rem;
				font-size: 2.5rem;
				font-weight: 300;
				color: #7b61ff;
			}
		}

		.content {
			display: grid;
			grid-template-columns: 1.1fr 0.9fr;

			.top-emotes {
				background: #15151b;
				padding: 1.5rem;
				width: 22.5rem;
				font-family: "Syne ExtraBold", sans-serif;

				h3 {
					margin-bottom: 1.25rem;
					font-size: 1.6rem;
					font-weight: 200;
					text-align: center;

					span {
						color: #ff3f7f;
					}
				}

				.emote-row {
					display: flex;
					align-items: center;
					gap: 1.4rem;
					margin-bottom: 1.75rem;
					.rank {
						width: 2rem;
						font-size: 1.6rem;
						font-weight: 400;
						text-align: center;
					}

					.emote-box {
						width: 2.2rem;
						height: 2.2rem;
					}

					.emote-name {
						padding: 0.35rem 0.7rem;
						background: #ff3f7f;
						color: #000000;
						font-size: 1.05rem;
						font-weight: 400;
					}
				}
			}

			.side-stats {
				background: #ffffff;
				color: #000000;
				padding: 4rem;

				display: flex;
				flex-direction: column;
				gap: 1.4rem;

				.stat {
					display: flex;
					flex-direction: column;

					.title {
						font-size: 1rem;
						font-weight: 400;
					}

					.number {
						font-size: 1rem;
						font-weight: 400;
						line-height: 1;

						&.pink {
							color: #ff3f7f;
						}
					}
				}
			}
		}
	}

	@media (max-width: 600px) {
		.page {
			padding: 1rem;
		}

		.recap {
			transform: scale(0.85);
			transform-origin: top center;
			width: 40rem;
		}
	}

	@media (max-width: 550px) {
		.recap {
			transform: scale(0.6);
		}
	}

	@media (max-width: 420px) {
		.recap {
			transform: scale(0.6);
		}
	}

	@media (max-width: 360px) {
		.recap {
			transform: scale(0.7);
		}
	}
</style>
