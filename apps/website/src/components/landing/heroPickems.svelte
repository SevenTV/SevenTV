<script lang="ts">
	import Button from "../input/button.svelte";
	import { Minus } from "phosphor-svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { type Badge } from "$/gql/graphql";
	import BadgeComponent from "../badge.svelte";
	import HeroPepe from "$/components/pickems/hero-pepe.svelte";
	// import PickemsHeaderBench from "$assets/pickems_header_bench.png";

	async function queryCosmetics() {
		let res = await gqlClient()
			.query(
				graphql(`
					query GetPickemsBadges {
						badges {
							badges {
								__typename
								id
								name
								description
								tags
								images {
									__typename
									url
									mime
									size
									scale
									width
									height
									frameCount
								}
								createdById
								updatedAt
								searchUpdatedAt
							}
						}
					}
				`),
				{},
			)
			.toPromise();
		return res.data;
	}
	let badges = $state<Badge[]>([]);

	const badgeIds = new Set([
		"01JJQEA3655687JWHG7P9CV3W8",
		"01JJQECTG04J5J6QE1BCATE6JN",
		"01JJQEDT21JXF1JM4F1P805VTK",
		"01JJQEENE6CJ6KR70CBCF39ACN",
	]);

	$effect(() => {
		queryCosmetics().then((res) => {
			badges = res?.badges.badges.filter((b: { id: string }) => badgeIds.has(b.id)) ?? [];
		});
	});
</script>

<div class="layout">
	<section class="hero">
		{#if badges.length > 0}
			<div class="badge-preview hide-on-mobile">
				<HeroPepe />
				<div class="badges">
					{#each badges as badge}
						<BadgeComponent enableDialog size={48} {badge} />
					{/each}
				</div>
			</div>
		{/if}
		<div class="hero-content centered">
			<div class="top-info hide-on-mobile">
				<p>35 STREAMERS</p>
				<Minus />
				<p>7TV HOSTED TOURNAMENT</p>
				<Minus />
				<p>3 DAY EVENT</p>
			</div>
			<div class="content">
				<div>
					<h1>Place Pick’ems</h1>
					<h1>Win Badges & Paints.</h1>
				</div>
				<div class="description">
					<p>xQc, OhnePixel, JasonTheWeen and 32 other streamers go head to head</p>
					<p>in a 3-day Single Elimination 7TV Hosted CS2 tournament.</p>
				</div>
			</div>
			<div class="buttons">
				<Button primary style="font-size: 1em; background-color: #EFDFFF;" href="/store/pickems"
					>Learn More</Button
				>
			</div>
		</div>
		<div class="header-image" style="margin-top: -2.5rem;">
			<!-- <img src={PickemsHeaderBench} alt="Header Bench" /> -->
		</div>
	</section>
</div>

<style lang="scss">
	.layout {
		margin: 3rem;
	}

	.top-info {
		opacity: 50%;
		top: -4rem;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		position: relative;

		p {
			margin: 0;
		}
	}
	.description {
		opacity: 50%;
	}

	.header-image {
		object-fit: contain;
		justify-content: center;
		z-index: 1;
		position: absolute;
		pointer-events: none;
		bottom: 0;
		img {
			display: inline;
			width: 100%;
			position: relative;
			margin-bottom: -13.5%;
			height: 100%;
		}
	}
	.hero {
		display: flex;
		justify-content: center;
		gap: 2rem;
		flex-wrap: wrap;
		padding: 5.5rem;
		min-height: min(50rem, 80vh);
		margin-bottom: 10rem;

		position: relative;
		z-index: 1;

		&::before {
			content: "";
			position: absolute;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			border-radius: 1rem;

			z-index: -1;

			background: radial-gradient(
					76.09% 165.33% at 50% 146.72%,
					#e8aa00 19.98%,
					rgba(59, 0, 137, 0.77) 68%,
					rgba(48, 0, 112, 0.09) 100%
				),
				#1111110a;

			mask-image: radial-gradient(
				180% 80% at 0% 100%,
				rgba(white, 1) 0%,
				rgba(white, 0.7) 25%,
				rgba(white, 0.5) 56%,
				rgba(white, 0.2) 79%,
				transparent 100%
			);
			mask-size: 100% 400%;
			animation: fade-in 0.5s linear forwards;
		}

		@keyframes fade-in {
			from {
				mask-position: 0 0;
			}
			to {
				mask-position: 0 100%;
			}
		}
	}

	.hero-content {
		margin-inline: auto;
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 1.25rem;
		padding: 2rem;

		.content {
			display: inline-grid;
			gap: 4rem;
			h1 {
				font-family: "Inter", sans-serif;
				font-size: 5rem;
				font-weight: 700;
				letter-spacing: 0.05rem;
				word-break: break-word;
				opacity: 0.9;
				background: linear-gradient(0deg, #fff 24.23%, rgba(255, 255, 255, 0.61) 100%);
				background-clip: text;
				-webkit-background-clip: text;
				-webkit-text-fill-color: transparent;
				text-align: center;
			}

			p {
				font-size: 1.125rem;
				font-weight: 400;
				line-height: 1.6rem;
			}
		}

		.buttons {
			display: inline-flex;
		}
	}

	.badge-preview {
		position: absolute;
		display: flex;
		justify-content: end;
		align-items: center;
		top: 0;
		right: 1rem;
		@media screen and (max-width: 1300px) {
			display: none;
		}

		.badges {
			display: flex;
			gap: 1rem;
			padding: 1rem;
			background: hsla(0deg, 0%, 40%, 20%);
			backdrop-filter: blur(2rem);
			border-radius: 0.5rem;
			z-index: 2;
			position: relative;
			top: 1rem;
		}
	}

	@media screen and (max-width: 960px) {
		.hero {
			padding: 2rem;
			margin-bottom: 0;
		}

		.layout {
			margin: 0;
		}
		.header-image {
			bottom: 0;
			width: 80%;
		}

		.hero-content .content {
			h1 {
				font-size: 1.8rem;
			}

			p {
				font-size: 1rem;
			}
		}
	}
</style>
