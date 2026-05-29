<script lang="ts">
	import SevenTVLogo from "../../components/icons/logo.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import type { Maybe, Paint } from "$/gql/graphql";
	import PaintComponent from "$/components/paint.svelte";
	import Global from "$/components/recap/global.svelte";
	import User from "$/components/recap/user.svelte";
	import { user } from "$/lib/auth";

	type RecapMode = "global" | "user";

	const potatbotatPaint = "01GZNT2FTG0006PK9PVYBXQ6KD";
	const yearPaint = "01HVJ2RVP800010B69FDA36C2V";
	let yearPaintData = $state<Maybe<Paint>[]>([]);
	let potatbotatPaintData = $state<Maybe<Paint>[]>([]);
	let potatData = $state<any>("");
	let TwitchUserId = $state<string | null>("");
	let PickedRecapMode = $state<RecapMode>("global");
	let isUserLoggedIn = $state(false);
	let cosmetics = $state<{ paints: any[]; badges: any[] }>({
		paints: [],
		badges: [],
	});

	import { onMount } from "svelte";
	let isMobile = $state(false);

	function checkMobile() {
		isMobile = window.innerWidth < 900;
	}

	onMount(() => {
		checkMobile();
		window.addEventListener("resize", checkMobile);
		return () => window.removeEventListener("resize", checkMobile);
	});

	async function getUserTwitchId(id: string) {
		const res = await gqlClient()
			.query(
				graphql(`
					query GetUserTwitchId($id: Id!) {
						users {
							user(id: $id) {
								connections {
									platform
									platformId
									platformUsername
								}
							}
						}
					}
				`),
				{ id },
			)
			.toPromise();
		if (res.error || !res.data?.users?.user?.connections) {
			throw res.error || new Error("User Twitch ID not found");
		}
		return res.data.users.user.connections.find(
			(conn: { platform: string }) => conn.platform === "TWITCH",
		)?.platformId;
	}

	async function loadPotatData(id: string) {
		if (!id) id = "0";
		const res = await fetch(`https://api.potat.app/wrapped/2025/7tv?id=${id}`);
		if (!res.ok) {
			throw new Error("Failed to fetch PotatBotat stats");
		}
		const data = await res.json();
		return data;
	}

	async function getAllCosmetics() {
		const res = await gqlClient()
			.query(
				graphql(`
					query Paints {
						paints {
							paints {
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
						badges {
							badges {
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
				`),
				{},
			)
			.toPromise();
		if (res.error || !res.data?.paints?.paints || !res.data?.badges?.badges) {
			throw res.error || new Error("Paints or badges not found");
		}

		return {
			paints: res.data.paints.paints ?? [],
			badges: res.data.badges.badges ?? [],
		};
	}

	$effect(() => {
		let stvUserId: string | null = null;
		(async () => {
			cosmetics = await getAllCosmetics();
			let potatPaint = cosmetics.paints.find(
				(paint: { id: string }) => paint.id === potatbotatPaint,
			);
			let yearPaintItem = cosmetics.paints.find((paint: { id: string }) => paint.id === yearPaint);
			yearPaintData = yearPaintItem ? [yearPaintItem as Maybe<Paint>] : [];
			potatbotatPaintData = potatPaint ? [potatPaint as Maybe<Paint>] : [];
			if ($user) {
				isUserLoggedIn = true;
				stvUserId = $user.id;
				TwitchUserId = (await getUserTwitchId(stvUserId)) ?? null;
			}
			potatData = (await loadPotatData(TwitchUserId!)) ?? null;
		})();
	});
</script>

<!-- <section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<SevenTVLogo size={isMobile ? 90 : 248} style="color: white; " />
			<p class="long-line">|</p>
			<div class="year-div">
				<h1 class="recap year" style="margin-bottom: -0.3em;">20</h1>
				<h1 class="recap year">25</h1>
			</div>
		</div>
		<h1 class="recap title" style="color:{PickedRecapMode === 'global' ? '#a6ff96' : '#ff3f7f'}">RECAP</h1>
		<h1 class="recap x">X</h1>
		<h1 class="recap x">
			{#if potatbotatPaintData[0]}
				<PaintComponent
					paint={potatbotatPaintData[0]}
					style="margin-top: 0.5rem;font-size: 1.3rem;"
				>
					PotatBotat
				</PaintComponent>
			{/if}
		</h1>
		<div class="side-by-side section-buttons">
			<button
				class="recap global {PickedRecapMode === 'global' ? 'active' : ''}"
				style={isUserLoggedIn ? "" : "margin-left: 2rem;"}
				onclick={() => (PickedRecapMode = "global")}
			>
				GLOBAL
			</button>
			{#if potatData && isUserLoggedIn}
				<button
					class="recap user {PickedRecapMode === 'user' ? 'active' : ''}"
					onclick={() => (PickedRecapMode = "user")}>USER</button
				>
			{:else if isUserLoggedIn}
				<button class="recap user loading" disabled>USER</button>
			{/if}
		</div>
	</header>
</section>

{#if potatData}
	<h1 class="recap global" style="text-align: center;font-size: 1rem; color: #aaa;">
		*Some data given by PotatBotat is tracked across channels using the bot and may not reflect your
		total emote usage on Twitch.
	</h1>
	{#if PickedRecapMode === "global"}
		<Global {potatData} {cosmetics} />
	{:else if PickedRecapMode === "user"}
		<User {potatData} {cosmetics} />
	{/if}
{/if} -->

<section class="hero">
	<header class="top">
		<h1 class="recap title" style="color:{PickedRecapMode === 'global' ? '#a6ff96' : '#ff3f7f'}">
			Coming soon ...
		</h1>
	</header>
</section>

<style lang="scss">
	.hero {
		position: relative;
		min-height: 60rem;
		color: #fff;
		background-color: #1d1d1d;
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
		margin-bottom: 10rem;
	}

	// Responsive styles
	@media (max-width: 1500px) {
		.recap {
			font-size: 5rem;
			&.title {
				font-size: 4rem;
			}
		}
		.hero {
			min-height: 40rem;
		}
		.top {
			margin-top: 6rem;
			margin-left: 1.5rem;
		}
	}

	@media (max-width: 900px) {
		.recap {
			font-size: 3rem;
			&.title {
				font-size: 2.2rem;
			}
		}
		.hero {
			min-height: 25rem;
		}
		.top {
			margin-top: 3rem;
			margin-left: 0.5rem;
		}
	}

	@media (max-width: 600px) {
		.recap {
			font-size: 2rem;
			&.title {
				font-size: 1.2rem;
			}
		}
		.hero {
			min-height: 15rem;
		}
		.top {
			margin-top: 1.5rem;
			margin-left: 0;
		}
	}
</style>
