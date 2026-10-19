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
	}

	const GlobalDataPaintsSaved: Array<Maybe<UserSavedEntry>> = [
		{
			id: "01JFQV4W300XYMWDJW7E1TVY34",
			name: "Winter Snowfall",
			count: 806895,
		},
		{
			id: "01JEDE45FTB7XTQGRFXP0BA2PT",
			name: "Domos 2024",
			count: 694686,
		},
		{
			id: "01JEY00EDNVW20AWX2NPG4HTNF",
			name: "NNYS 2024",
			count: 683837,
		},
		{
			id: "01JK6NVC5W6VS5D0KTJXF6SQRC",
			name: "Molten Core",
			count: 612212,
		},
		{
			id: "01JHXJCX6WPJR5ETEYRDP6WVYH",
			name: "CS Gold",
			count: 585498,
		},
		{
			id: "01JM3P7C1EJ4MH69BDNV0670JZ",
			name: "Lovers",
			count: 507340,
		},
		{
			id: "01JRZJN6CKSPDT7JB09YNNY2SV",
			name: "Easter Gifting 2025",
			count: 479601,
		},
		{
			id: "01HGV83AY80000421YCKFBCF2F",
			name: "NNYS 2023",
			count: 445963,
		},
		{
			id: "01J5K5Y85000026B027ATG06TR",
			name: "demeDeme",
			count: 370451,
		},
		{
			id: "01H9904F78000BVQ7MEDFXVNTK",
			name: "Sunflower",
			count: 366281,
		},
	];
	const GlobalDataBadgesSaved: Array<Maybe<UserSavedEntry>> = [
		{
			id: "01GAF9E5HG000E8VNG1S1RMTBE",
			name: "7TV Subscriber - 9 Months",
			count: 380015,
		},
		{
			id: "01GAF9BTB8000E8VNG1S1RMTBD",
			name: "7TV Subscriber - 6 Months",
			count: 341993,
		},
		{
			id: "01GAF994D8000E8VNG1S1RMTBC",
			name: "7TV Subscriber - 3 Months",
			count: 314643,
		},
		{
			id: "01GAF95ZTG000E8VNG1S1RMTBB",
			name: "7TV Subscriber - 2 Months",
			count: 306649,
		},
		{
			id: "01GAF8RWW8000E8VNG1S1RMTBA",
			name: "7TV Subscriber - 1 Month",
			count: 299940,
		},
		{
			id: "01JEY956QEPVSFKH7KWS9V6HMQ",
			name: "NNYS 2024",
			count: 258894,
		},
		{
			id: "01JFQV88ENDHWA3TXJHVVT0BQJ",
			name: "X-MAS 2024 (ANIMATED)",
			count: 206450,
		},
		{
			id: "01JFQTXCRE600BMC0214W6KHMF",
			name: "Enchanted Glint",
			count: 176270,
		},
		{
			id: "01JM3F4N2Y65KKHDGWDEDFTJNA",
			name: "Valentine Gifter",
			count: 175852,
		},
		{
			id: "01JFQTX2N45DSA992YKT3143VT",
			name: "Enchanted Diamond",
			count: 167400,
		},
	];

	const { cosmetics }: Props = $props();
	const paintIdSet = new Set(GlobalDataPaintsSaved.map((p) => p?.id));
	const paintCountMap = new Map<string, number>(
		GlobalDataPaintsSaved.filter((p): p is UserSavedEntry => !!p?.id).map((p) => [p.id, p.count]),
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
	const badgeIdSet = new Set(GlobalDataBadgesSaved.map((b) => b?.id));
	const badgeCountMap = new Map<string, number>(
		GlobalDataBadgesSaved.filter((b): b is UserSavedEntry => !!b?.id).map((b) => [b.id, b.count]),
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

	if (GlobalDataPaintsSaved.length === 0 && GlobalDataBadgesSaved.length > 0) {
		PickedRecapMode = "badges";
	}
</script>

<section class="hero">
	<header class="top">
		<h1 class="recap title1">TOP FAVOURITE</h1>
		<h1 class="recap title2">COSMETICS*</h1>
		<div style="margin-top: 3rem;">
			<p class="recap subtitle">Cosmetics? We all love them.</p>
			<p class="recap subtitle">Here are the top used</p>
			<p class="recap subtitle">cosmetics of this year</p>
		</div>
		<div class="section-buttons">
			{#if GlobalDataPaintsSaved && GlobalDataPaintsSaved.length > 0}
				<button
					class="recap global {PickedRecapMode === 'paints' ? 'active' : ''}"
					onclick={() => (PickedRecapMode = "paints")}
				>
					Paints
				</button>
			{/if}
			{#if GlobalDataBadgesSaved && GlobalDataBadgesSaved.length > 0}
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
						<div title={item.name}>
							<PaintComponent
								dialogWidth={60}
								paint={item.paint}
								enableDialog
								style="font-size: 1rem; padding: 1rem;"
							>
								{item.name}
							</PaintComponent>
						</div>
						<span title={item.name} class="name"></span>
						<span class="score">{item.score.toLocaleString()}</span>
					</div>
				{/each}
			{:else if PickedRecapMode === "badges"}
				{#each leaderboardBadges.slice(0, 10) as item}
					<div class="row badges">
						<span class="rank">{item.rank}.</span>
						<div title={item.name}>
							<BadgeComponent
								badge={item.badge}
								enableDialog
								style="margin-top: 0.5rem;"
								size={48}
							/>
						</div>
						<span title={item.name} class="name">{item.name}</span>
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
		&.subtitle {
			font-size: 1.5rem;
			font-weight: 1000;
			text-align: center;
			font-family: "Syne Bold", sans-serif;
			color: #ffffff;
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
		grid-template-columns: 14px 310px 1fr auto;
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
			width: 40rem;
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
