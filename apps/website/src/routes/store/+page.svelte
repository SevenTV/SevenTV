<script lang="ts">
	import Spinner from "$/components/spinner.svelte";
	import BadgeProgressComponent from "$/components/store/badge-progress.svelte";
	import Banner from "$/components/store/banner.svelte";
	import Benefits from "$/components/store/benefits.svelte";
	import MonthlyPaints from "$/components/store/monthly-paints.svelte";
	import PersonalEmotes from "$/components/store/personal-emotes.svelte";
	import YourSub from "$/components/store/your-sub.svelte";
	import { graphql } from "$/gql";
	import { EmoteSetKind, type BadgeProgress, type Paint } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { PaintBrush, Seal, Smiley, UserCircle } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	let subbed = $state(false);

	async function queryStore() {
		let res = await gqlClient().query(
			graphql(`
				query StoreData {
					store {
						badgeProgress {
							currentBadgeId
							nextBadge {
								badgeId
								percentage
								daysLeft
							}
						}
						monthlyPaints {
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
			`),
			{},
		);

		return res.data?.store;
	}

	let storeData = $derived(queryStore());
</script>

<svelte:head>
	<title>{$t("common.subscriptions", { values: { count: 1 } })} - {$t("page_titles.suffix")}</title>
</svelte:head>

<!-- All things called grid here aren't actually css grids -->
<Banner
	title={subbed
		? $t("pages.store.subscription.banner_title_subbed")
		: $t("pages.store.subscription.banner_title_unsubbed")}
	subtitle={subbed
		? $t("pages.store.subscription.banner_subtitle_subbed")
		: $t("pages.store.subscription.banner_subtitle_unsubbed")}
>
	<div class="banner-icons hide-on-mobile">
		<PaintBrush size="1.8rem" />
		<UserCircle size="1.8rem" />
		<Seal size="1.8rem" />
		<!-- <Ticket size="1.8rem" /> -->
		<Smiley size="1.8rem" />
	</div>
</Banner>
<div class="grid">
	{#if !subbed}
		<Benefits />
	{/if}
	<div class="top-grid">
		{#await storeData}
			<Spinner />
		{:then storeData}
			{#if storeData}
				<div class="subgrid">
					<YourSub bind:subbed />
					<BadgeProgressComponent progress={storeData.badgeProgress as BadgeProgress} />
				</div>
				<MonthlyPaints paints={storeData.monthlyPaints as Paint[]} />
			{/if}
		{/await}
	</div>
	<PersonalEmotes />
	<!-- <div class="three-grid">
		<EmoteRaffle />
		<PersonalEmotes />
		<TopGifters />
	</div> -->
	{#if subbed}
		<Benefits />
	{/if}
	<!-- <Faq /> -->
</div>

<style lang="scss">
	.banner-icons {
		padding: 0 2.75rem;

		display: flex;
		gap: 3.5rem;
		flex-wrap: wrap;
		align-items: center;
		justify-content: center;
	}

	.grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		flex-wrap: wrap;

		max-width: 70rem;
		margin-top: 1rem;
		margin-inline: auto;
	}

	.top-grid {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;

		& > .subgrid {
			flex-grow: 1;

			display: flex;
			flex-direction: column;
			gap: 1rem;
			flex-wrap: wrap;
		}
	}

	// .three-grid {
	// 	display: flex;
	// 	gap: 1rem;
	// 	flex-wrap: wrap;
	// }
</style>
