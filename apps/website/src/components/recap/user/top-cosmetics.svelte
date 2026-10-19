<script lang="ts">
	import type { Maybe, Paint, Badge } from "$/gql/graphql";
	import PaintComponent from "$/components/paint.svelte";
	import BadgeComponent from "$/components/badge.svelte";

	type UserSavedEntry = {
		id: string;
		name?: string; 
		count: number;
	};
	type CosmeticMode = "paints" | "badges";

	interface Props {
		cosmetics: {
			paints: Paint[];
			badges: Badge[];
		};
		UserDataPaintsSaved: Array<Maybe<UserSavedEntry>>;
		UserDataBadgesSaved: Array<Maybe<UserSavedEntry>>;
	}

	const { cosmetics, UserDataPaintsSaved, UserDataBadgesSaved }: Props = $props();
	const paintIdSet = new Set(UserDataPaintsSaved.map((p) => p?.id));
	const paintCountMap = new Map<string, number>(
		UserDataPaintsSaved.filter((p): p is UserSavedEntry => !!p?.id).map((p) => [p.id, p.count]),
	);
	const allPaintsData: Paint[] = cosmetics.paints.filter((paint) => paintIdSet.has(paint.id));
	const leaderboardPaints = allPaintsData
		.sort((a, b) => (paintCountMap.get(b.id) ?? 0) - (paintCountMap.get(a.id) ?? 0))
		.map((paint, index) => ({
			rank: index + 1,
			id: paint.id,
			name: paint.name ?? "Unknown",
			alias: paint.name ?? "unknown",
			paint: paint,
			score: paintCountMap.get(paint.id) ?? 0,
		}));

	const badgeIdSet = new Set(UserDataBadgesSaved.map((b) => b?.id));
	const badgeCountMap = new Map<string, number>(
		UserDataBadgesSaved.filter((b): b is UserSavedEntry => !!b?.id).map((b) => [b.id, b.count]),
	);
	const allBadgesData: Badge[] = cosmetics.badges.filter((badge) => badgeIdSet.has(badge.id));
	const leaderboardBadges = allBadgesData
		.sort((a, b) => (badgeCountMap.get(b.id) ?? 0) - (badgeCountMap.get(a.id) ?? 0))
		.map((badge, index) => ({
			rank: index + 1,
			id: badge.id,
			name: badge.name ?? "Unknown",
			alias: badge.name ?? "unknown",
			badge: badge,
			score: badgeCountMap.get(badge.id) ?? 0,
		}));

	let PickedRecapMode = $state<CosmeticMode>("paints");

	if(UserDataPaintsSaved?.length === 0 && UserDataBadgesSaved?.length > 0) {
		PickedRecapMode = "badges";
	}
</script>

<section class="hero">
	<header class="top">
		<h1 class="recap title1">YOUR FAVOURITE</h1>
		<h1 class="recap title2">COSMETICS*</h1>
		<div class="section-buttons">
			{#if UserDataPaintsSaved && UserDataPaintsSaved?.length > 0}
				<button
					class="recap global {PickedRecapMode === 'paints' ? 'active' : ''}"
					onclick={() => (PickedRecapMode = "paints")}
				>
					Paints
				</button>
			{/if}
			{#if UserDataBadgesSaved && UserDataBadgesSaved?.length > 0}
				<button
					class="recap user {PickedRecapMode === 'badges' ? 'active' : ''}"
					onclick={() => (PickedRecapMode = "badges")}>Badges</button
				>
			{/if}
		</div>
		<div class="leaderboard">
			{#if PickedRecapMode === "paints"}
				{#each leaderboardPaints.slice(0, 10) as item}
					<div class="row">
						<span class="rank">{item.rank}.</span>
						<div>
							<PaintComponent
								dialogWidth={60}
								paint={item.paint}
								enableDialog
								style="font-size: 1rem; padding: 1.1rem;"
							>
								{item.name}
							</PaintComponent>
						</div>
						<span class="name"></span>
						<span class="score">{item.score.toLocaleString()}</span>
					</div>
				{/each}
			{:else if PickedRecapMode === "badges"}
				{#each leaderboardBadges.slice(0, 10) as item}
					<div class="row badges">
						<span class="rank">{item.rank}.</span>
						<div>
							<BadgeComponent
								badge={item.badge}
								enableDialog
								style="margin-top: 0.5rem;"
								size={48}
							/>
						</div>
						<span class="name">{item.name}</span>
						<span class="score">{item.score.toLocaleString()}</span>
					</div>
				{/each}
			{/if}
		</div>
	</header>
</section>

<style lang="scss">
	.section-buttons {
		margin-top: 5rem;
		display: flex;
		margin-bottom: 5rem;
		gap: 8rem;
	}
	.recap {
		font-size: 7rem;
		margin: 0;
		font-family: "BBH Bartleby", sans-serif;
		font-weight: 200;

		&.title1 {
			color: #ffffff;
			font-size: 2.5rem;

			letter-spacing: 0.3rem;
		}
		&.title2 {
			color: #ff3f7f;
			font-size: 5.5rem;

			letter-spacing: 0.3rem;
		}
		&.global {
			font-size: 2.5rem;
			margin-right: 1rem;
		}

		&.active {
			color: #836fff;
		}

		&.user {
			font-size: 2.5rem;
		}
	}
	.hero {
		position: relative;
		min-height: 60rem;
		color: #fff;

		background-color: #1d1d1d;
		background-image: url("../../../../assets/user-recap-shapes2.png");
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
		margin-bottom: 5rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		margin-left: 3rem;
	}

	.leaderboard {
		width: 64rem;
		background: #05002c;
		padding: 2rem;
		margin: 1rem;
		display: flex;
		flex-direction: column;
		gap: 14px;
		border: 1rem solid #ffffff;
		font-family: "BBH Bartleby", sans-serif;
	}
	.row {
		display: grid;
		grid-template-columns: 31px 400px 1fr auto;
		align-items: center;
		gap: 30px;
		color: #ffffff;
		font-weight: 100;

		&.badges {
			grid-template-columns: 31px 40px 1fr auto;
		}

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

	@media (max-width: 1500px) {
		.section-buttons {
			margin-top: 2rem;
			display: flex;
			margin-bottom: 2rem;
			gap: 0rem;
		}
		.recap {
			&.title1 {
				font-size: 2rem;
			}
			&.title2 {
				font-size: 5rem;
			}
			&.global {
				font-size: 1rem;
			}
			&.user {
				font-size: 1rem;
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
			width: 100%;
			max-width: 32rem;
			padding: 1.2rem;
		}
		.row {
			grid-template-columns: 24px 12rem 1fr auto;
			gap: 16px;

			&.badges {
				grid-template-columns: 31px 70px 1fr auto;
			}
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
			max-width: 100%;
			padding: 0.7rem;
			border-width: 0.5rem;
		}
		.row {
			gap: 10px;
			grid-template-columns: 40px 59px 1fr auto;
			&.badges {
				grid-template-columns: 31px 70px 1fr auto;
			}
			.name,
			.score,
			.rank {
				font-size: 0.7rem;
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
		}
		.hero {
			min-height: 15rem;
		}
		.top {
			margin-top: 1rem;
			margin-left: 0;
		}
		.leaderboard {
			width: 22rem;
			padding: 0.3rem;
			border-width: 0.3rem;
			gap: 8px;
		}
		.row {
			gap: 6px;
			grid-template-columns: 40px 59px 1fr auto;
			&.badges {
				grid-template-columns: 31px 70px 1fr auto;
			}
		}
	}
</style>
