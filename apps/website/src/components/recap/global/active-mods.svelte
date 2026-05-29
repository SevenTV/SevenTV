<script lang="ts">
	import { gqlClient } from "$/lib/gql";
	import { type User } from "$/gql/graphql";
	import UserProfilePicture from "$/components/user-profile-picture.svelte";
	import UserName from "$/components/user-name.svelte";
	const leaderboard = [
		{ rank: 1, id: "01F46N0YZ80005W0EZC5BGNE80", score: 99069 },
		{ rank: 2, id: "01FMJ3Z6R0000D6HG894PC6RKJ", score: 54735 },
		{ rank: 3, id: "01H95K0RV80003CY09NFH3G7V3", score: 43050 },
		{ rank: 4, id: "01FFG8NP580007P57XYW0BHHB3", score: 35708 },
		{ rank: 5, id: "01F6N9TF5G0006SZ7ZW5FH370R", score: 31875 },
		{ rank: 6, id: "01F74DWQMR0005C7FW3P0F45Y5", score: 31575 },
		{ rank: 7, id: "01F6M9ZTFR0002B6P5MWZ4FSEB", score: 20828 },
		{ rank: 8, id: "01FEZ4M12R00031H59C3CN17SK", score: 18520 },
		{ rank: 9, id: "01EZPK137G0009D0SFZ9WDAXV7", score: 18216 },
		{ rank: 10, id: "01F729AGSR00008A1TQ59DB1D2", score: 16394 },
	];
	let users = $state<User[]>([]);

	function buildUsersCreatedAtQuery(ids: readonly string[]) {
		const fields = ids
			.map(
				(id, i) => `
				u${i}: user(id: "${id}") {
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
							highestRoleColor {
								hex
							}		
				}
			`,
			)
			.join("\n");

		const query = `
		query {
			users {
				${fields}
			}
		}
	`;

		return query;
	}

	async function getUsersCreatedAt(ids: readonly string[]) {
		const query = buildUsersCreatedAtQuery(ids);

		const res = await gqlClient().query(query, {}).toPromise();

		if (res.error || !res.data?.users) {
			throw res.error || new Error("Users not found");
		}

		return ids.map((_, i) => res.data.users[`u${i}`]);
	}

	$effect(() => {
		(async () => {
			const idsToFetch = leaderboard.slice(0, 6).map((item) => item.id);
			const idsToFetch2 = leaderboard.slice(6, 10).map((item) => item.id);
			users = await getUsersCreatedAt(idsToFetch);
			let users2 = await getUsersCreatedAt(idsToFetch2);
			users = users.concat(users2);
		})();
	});
</script>

<section class="hero">
	<header class="top">
		<h1 class="recap title1">MOST ACTIVE</h1>
		<h1 class="recap title2">MODS</h1>
		<div style="margin-top: 3rem;">
			<p class="recap subtitle">Our precious janitors did lots of</p>
			<p class="recap subtitle">work this year. Let's appreciate our</p>
			<p class="recap subtitle">top 10 of this year!</p>
		</div>
		<div class="leaderboard">
			{#each leaderboard as item}
				<div class="row">
					<span class="rank">{item.rank}.</span>
					<span class="name">
						<a class="profile" href="/users/{item.id}" target="_blank">
							{#if users.find((u) => u?.id === item?.id)}
								<UserProfilePicture
									user={users.find((u) => u?.id === item.id)}
									size={3 * 16}
									style="grid-row: 1 / -1"
								/>
								<span class="name">
									<UserName user={users.find((u) => u?.id === item?.id)!} />
								</span>
							{/if}
						</a>
					</span>
					<span class="score">{item.score.toLocaleString()}</span>
				</div>
			{/each}
		</div>
	</header>
</section>

<style lang="scss">
	.profile{
		display: flex;
		align-items: center;
		gap: 1rem;
		text-decoration: none;
		color: inherit;
	}
	.recap {
		font-size: 7rem;
		margin: 0;
		font-family: "BBH Bartleby", sans-serif;
		font-weight: 200;

		&.title1 {
			color: #000000;
			font-size: 2.5rem;

			letter-spacing: 0.3rem;
		}
		&.title2 {
			color: #000000;
			font-size: 7.5rem;

			letter-spacing: 0.3rem;
		}

		&.subtitle {
			font-size: 1.5rem;
			font-weight: 1000;
			text-align: center;
			font-family: "Syne Bold", sans-serif;
			color: #1d1d1d;
		}
	}
	.hero {
		position: relative;
		min-height: 60rem;
		color: #fff;

		background-color: #a6ff96;
		background-image: url("../../../../assets/recap-shapes.png");
		background-repeat: no-repeat;
		background-position: center center;
		background-size: cover;

		overflow: hidden;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.top {
		margin-top: 10rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		margin-left: 3rem;
	}

	.leaderboard {
		width: min(40rem, 95vw);
		max-width: 95vw;
		background: #836FFF;
		padding: 2rem;
		display: flex;
		margin-top: 2rem;
		margin-bottom: 2rem;
		flex-direction: column;
		gap: 14px;
		border: 1rem solid #000000;
		font-family: "BBH Bartleby", sans-serif;
	}
	.row {
		display: grid;
		grid-template-columns: 30px 23rem 1fr auto;
		align-items: center;
		gap: 30px;
		color: #ffffff;
		font-weight: 100;

		.rank {
			font-size: 1rem;
			opacity: 0.9;
			color: #000000;
		}

		.name {
			font-size: 1.1rem;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
			font-family: "Syne ExtraBold", sans-serif;
			font-weight: 900;
			letter-spacing: 0.1rem;
			color: #000000;
		}

		.score {
			font-size: 1rem;
			letter-spacing: 0.3px;
			color: #000000;
		}
	}

	@media (max-width: 1500px) {
		.recap {
			&.title1 {
				font-size: 2rem;
			}
			&.title2 {
				font-size: 5rem;
			}
			&.subtitle {
				font-size: 1.2rem;
			}
		}
		.hero {
			min-height: 40rem;
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
		.leaderboard {
			width: min(30rem, 98vw);
			padding: 1.2rem;
			border-width: 0.7rem;
		}
		.row {
			grid-template-columns: 24px 12rem 1fr auto;
			gap: 16px;
			.name {
				font-size: 0.7rem;
			}
			.rank {
				font-size: 0.7rem;
			}
		}
	}

	@media (max-width: 900px) {
		.recap {
			&.title1 {
				font-size: 1.3rem;
			}
			&.title2 {
				font-size: 2.5rem;
			}
			&.subtitle {
				font-size: 1rem;
			}
		}
		.hero {
			min-height: 25rem;
		}
		.top {
			margin-top: 2rem;
			margin-left: 0.5rem;
		}
		.leaderboard {
			width: 100%;
			padding: 0.7rem;
			border-width: 0.4rem;
		}
		.row {
			grid-template-columns: 35px 1fr auto;
			gap: 8px;
			.name {
				font-size: 0.95rem;
			}
			.score {
				font-size: 0.9rem;
			}
		}
	}

	@media (max-width: 600px) {
		.recap {
			&.title1 {
				font-size: 1rem;
			}
			&.title2 {
				font-size: 1.5rem;
			}
			&.subtitle {
				font-size: 0.9rem;
			}
		}
		.hero {
			min-height: 15rem;
		}
		.top {
			margin-top: 1rem;
			margin-left: 0;
		}
		.leaderboard {
			width: 100%;
			padding: 0.3rem;
			border-width: 0.2rem;
		}
		.row {
			grid-template-columns: 35px 1fr auto;
			gap: 4px;
			.name {
				font-size: 0.8rem;
			}
			.score {
				font-size: 0.8rem;
			}
		}
	}
</style>
