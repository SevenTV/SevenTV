<script lang="ts">
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
	import nnysPlaceholder from "$assets/nnys.webp?url";
	import { isXmasEvent } from "$/lib/xmas";
	import moment from "moment";
	import { Minus } from "phosphor-svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { type Badge } from "$/gql/graphql";
	import BadgeComponent from "../badge.svelte";
	async function queryCosmetics() {
		let res = await gqlClient()
			.query(
				graphql(`
					query PickemsGetCosmetics {
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
	{#if isXmasEvent()}
		<section class="gifting hero-content">
			<div class="content">
				<h1>X-MAS SUB EVENT</h1>
				<p>
					Gift 1 sub to anyone this christmas and get a special badge!
					<br />
					<a href="/store">Gift a sub</a>
				</p>
			</div>
		</section>
	{/if}

	{#if moment("2024-12-20T16:00:00Z").isAfter()}
		<section class="hero nnys dark-theme">
			<div class="hero-content">
				<Logo size={8.5 * 16} />
				<div class="content">
					<h1>
						The Emote Platform <span class="for-all">for All</span>
					</h1>
					<p>
						Manage hundreds of emotes for your Twitch, Kick or YouTube channels with ease. Enhance
						your chatting experience.
					</p>
					<p>Vote on NNYS 2024 to get an exclusive paint, badge & emote set.</p>
				</div>
				<div class="buttons">
					<Button primary style="font-size: 1em;" href="https://www.nnys.live/">Vote now!</Button>
					<Button style="font-size: 1em;" href="#download">Download</Button>
				</div>
			</div>
			<img class="hero-image hide-on-mobile" src={nnysPlaceholder} alt="7TV" />
		</section>
	{:else}
		<section class="hero">
			{#if badges.length > 0}
				<div class="badge-preview">
					<img src="../assets/pickems_peepo_eye.svg" class="pepe-eye-right" />
					<img src="../assets/pickems_peepo_eye.svg" class="pepe-eye-left" />
					<!-- svelte-ignore a11y_missing_attribute -->
					<img src="../assets/pickems_peepo_hand.png" class="pepe-hand" />
					<!-- svelte-ignore a11y_missing_attribute -->
					<img src="../assets/pickems_peepo_no_eyes.svg" class="pepe-face" />
					<div class="badges">
						{#each badges as badge}
							<BadgeComponent enableDialog size={48} {badge}></BadgeComponent>
						{/each}
					</div>
				</div>
			{/if}
			<div class="hero-content centered">
				<div class="top-info">
					<p>16 STREAMERS</p>
					<Minus />
					<p>CS2 HOSTED TOURNAMENT</p>
					<Minus />
					<p>3 DAY EVENT</p>
				</div>
				<div class="content">
					<div>
						<h1>Place Pick’ems</h1>
						<h1>Win Badges & Paints.</h1>
					</div>
					<div class="description">
						<p>xQc, OhnePixel, JasonTheWeen and 13 other streamers go head to head</p>
						<p>in a 3-day Single Elimination 7TV Hosted CS2 tournament.</p>
					</div>
				</div>
				<div class="buttons">
					<Button
						primary
						style="font-size: 1em; background-color: #EFDFFF;"
						href="/landing/pickems_pass/about">Learn More</Button
					>
				</div>
			</div>
		</section>
		<div class="header-image" style="margin-top: -2.5rem;">
			<img src="../assets/pickems_header_bench.png" alt="Header Bench" />
		</div>
	{/if}
</div>

<style lang="scss">
	.layout {
		margin: 3rem;
	}
	.pepe-face {
		bottom: 1rem;
		position: absolute;
		width: 11rem;
		left: 3rem;
	}
	.pepe-eye-right {
		position: absolute;
		z-index: 1;
		width: 2.5rem;
		bottom: 4rem;
		left: 6rem;
	}
	.pepe-eye-left {
		position: absolute;
		z-index: 1;
		width: 2.5rem;
		left: 10.5rem;
		bottom: 4rem;
	}
	.pepe-hand {
		position: absolute;
		width: 4rem;
		left: 15.5rem;
		z-index: 3;
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
		display: flex;
		z-index: 1;
		bottom: 12rem;
		position: relative;
		img {
			display: inline;
			width: 1400px;
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

		position: relative;
		z-index: 1;

		&.nnys {
			color: var(--text);

			&::before {
				background: url("$assets/nnys-background.webp?url");
				background-size: cover;

				content: "";
				position: absolute;
				top: 0;
				left: 0;
				right: 0;
				bottom: 0;
				border-radius: 1rem;

				z-index: -1;

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

			.hero-content {
				background-color: #00000027;
				backdrop-filter: blur(1rem);
				border-radius: 1rem;
			}
		}

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
				mask-position: 0% 0%;
			}
			to {
				mask-position: 0% 100%;
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
				.for-all {
					letter-spacing: calc(0.15rem + 2px);
					font-size: 2.65rem;
					-webkit-text-stroke-width: 1.5px;
					-webkit-text-stroke-color: var(--text);
				}
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

	.hero-image {
		width: 100%;
		min-width: 15rem;
		max-width: 25rem;

		margin-inline: auto;
	}
	.badge-preview {
		position: absolute;
		display: flex;
		justify-content: end;
		align-items: center;
		top: 0rem;
		right: 1rem;

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
	.gifting {
		align-items: center;
		text-align: center;
	}

	@media screen and (max-width: 960px) {
		.hero {
			padding: 2rem;
		}

		.hero-content .content {
			h1 {
				font-size: 1.8rem;

				.for-all {
					font-size: 2rem;
				}
			}

			p {
				font-size: 1rem;
			}
		}
	}
</style>
