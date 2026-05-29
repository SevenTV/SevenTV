<script lang="ts">
	import { gqlClient } from "$/lib/gql";
	import { type User } from "$/gql/graphql";
	import UserProfilePicture from "$/components/user-profile-picture.svelte";
	import UserName from "$/components/user-name.svelte";
	const leaderboard = [
		{ rank: 1, name: "theconst4ntine2", id: "01JM0FFYD4CH8JDD12E2DY5C9P", score: 999 },
		{ rank: 2, name: "semmitell", id: "01FAJ17CE00005Z0WD0X9R5GXR", score: 927 },
		{ rank: 3, name: "lucasepicvt", id: "01HVYMYBX8000AVYEJBSJP9JAW", score: 927 },
		{ rank: 4, name: "wordsmybeloved", id: "01K3JQMWW2V3GHDW8HCDGXSSM8", score: 834 },
		{ rank: 5, name: "LT_Dangles", id: "01GYDKXGCG0007CK1ENYEVB6K0", score: 821 },
		{ rank: 6, name: "slowthu", id: "01HDM6KQZR00006CPS3WSNTFCJ", score: 738 },
		{ rank: 7, name: "wutwha", id: "01F6MV2SDG000BSPXHPYFZHMFG", score: 682 },
		{ rank: 8, name: "simonex93", id: "01HX1S3PCR0003S49WDMY4PQ40", score: 632 },
		{ rank: 9, name: "cracksonhead36", id: "01J3FCK6000006R1NJPSJHX11N", score: 603 },
		{ rank: 10, name: "speyll", id: "01FQF6RYMR000A7N4YK2NVMARX", score: 501 },
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
		<h1 class="recap title1">TOP EMOTE</h1>
		<h1 class="recap title2">UPLOADERS</h1>
		<div style="margin-top: 3rem;">
			<p class="recap subtitle">Without uploaders, 7TV won't have</p>
			<p class="recap subtitle">any emotes at all. Here are the</p>
			<p class="recap subtitle">top 10 uploaders of this year!</p>
		</div>
		<div class="leaderboard">
			{#each leaderboard as item}
				<div class="row">
					<span class="rank">{item.rank}.</span>
					<span class="name">
						<a class="profile" href="/users/{item.id}" style="color: white;" target="_blank">
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
	.profile {
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
			font-size: 5.5rem;
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

		background-color: rgb(255, 255, 255);
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
		background: #8576ff;
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
		}

		.name {
			font-size: 1.1rem;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
			font-family: "Syne ExtraBold", sans-serif;
			font-weight: 900;
			letter-spacing: 0.1rem;
		}

		.score {
			font-size: 1rem;
			letter-spacing: 0.3px;
		}
	}

	// Responsive styles
	@media (max-width: 1500px) {
		.hero {
			min-height: 40rem;
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
		.recap.title1 {
			font-size: 2rem;
		}
		.recap.title2 {
			font-size: 4rem;
		}
		.leaderboard {
			width: min(30rem, 98vw);
			padding: 1.2rem;
			border-width: 0.7rem;
		}
		.row {
			grid-template-columns: 24px 15rem 1fr auto;
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
		.hero {
			min-height: 30rem;
		}
		.top {
			margin-top: 2rem;
			margin-left: 0.5rem;
		}
		.recap.title1 {
			font-size: 1.3rem;
		}
		.recap.title2 {
			font-size: 2.2rem;
		}
		.recap.subtitle {
			font-size: 1rem;
		}
		.leaderboard {
			width: 100%;
			padding: 0.7rem;
			border-width: 0.4rem;
		}
		.row {
			        grid-template-columns: 35px 1fr auto;
			gap: 10px;
		}
	}

	@media (max-width: 600px) {
		.hero {
			min-height: 20rem;
		}
		.top {
			margin-top: 1rem;
			margin-left: 0;
		}
		.recap.title1 {
			font-size: 1rem;
		}
		.recap.title2 {
			font-size: 1.3rem;
		}
		.recap.subtitle {
			font-size: 0.9rem;
		}
		.leaderboard {
			width: 100%;
			padding: 0.4rem;
			border-width: 0.2rem;
		}
		.row {
			        grid-template-columns: 35px 1fr auto;
			gap: 6px;
		}
		.profile {
			gap: 0.5rem;
		}
	}
</style>
