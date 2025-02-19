<script lang="ts">
	import { ArrowSquareOut, Ticket, X } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import PaintComponent from "../paint.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { type Paint, type Badge } from "$/gql/graphql";
	import BadgeComponent from "../badge.svelte";
	import { user } from "$/lib/auth";
	import { onMount } from "svelte";
	import { fly } from "svelte/transition";

	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);
	let paintMouseOver = $state(false);
	let username = $derived($user?.mainConnection?.platformDisplayName ?? "Username");

	let dismissed = $state(window.localStorage.getItem("pickems-dismissed") ?? "false");
	function dismiss() {
		dismissed = "true";
		window.localStorage.setItem("pickems-dismissed", "true");
	}

	async function queryCosmetics() {
		let res = await gqlClient()
			.query(
				graphql(`
					query GetCosmetics {
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
					}
				`),
				{},
			)
			.toPromise();

		return res.data;
	}

	let badges = $state<Badge[]>([]);
	let paints = $state<Paint[]>([]);
	let paintIndex = $state(0);
	onMount(() => {
		const interval = setInterval(() => {
			paintIndex = (paintIndex + 1) % paints.length;
		}, 5000);
		return () => clearInterval(interval);
	});

	const badgeIds = new Set([
		"01JJQEA3655687JWHG7P9CV3W8",
		"01JJQECTG04J5J6QE1BCATE6JN",
		"01JJQEDT21JXF1JM4F1P805VTK",
		"01JJQEENE6CJ6KR70CBCF39ACN",
	]);

	const paintIds = new Set(["01JHXJH9C9MHN9FJMPNQB4YZ4N", "01JHXJCX6WPJR5ETEYRDP6WVYH"]);

	$effect(() => {
		queryCosmetics().then((res) => {
			badges = res?.badges.badges.filter((b) => badgeIds.has(b.id)) ?? [];
			paints = (res?.paints.paints.filter((p) => paintIds.has(p.id)) ?? []) as Paint[];
		});
	});
</script>

{#if dismissed !== "true"}
	<div class="pickems-header-layout">
		<div class="pickems-header">
			<div class="purchase-info">
				<p class="pre-title">7TV Hoster CS2 Tournament &#x2022 Feb. 29th - Mar. 2nd</p>
				<h2 class="title">Place your Pick'ems. Win Prizes.</h2>
				<div class="buttons">
					{#if hasPass}
						<Button primary href="https://pickems.tv">
							{#snippet iconRight()}
								<ArrowSquareOut />
							{/snippet}
							Place Pick'ems
						</Button>
					{:else}
						<Button href="/store" primary>
							{#snippet iconRight()}
								<Ticket />
							{/snippet}
							Purchase Pass
						</Button>
						<Button secondary href="https://pickems.tv">
							{#snippet iconRight()}
								<ArrowSquareOut />
							{/snippet}
							Learn More
						</Button>
					{/if}
				</div>
			</div>
			<div class="paint-preview hide-on-mobile">
				<div class="paint">
					{#each paints as paint, i}
						{#if paintIndex === i}
							<div
								class="paint-inner"
								in:fly={{ y: 100, duration: 500 }}
								out:fly={{ y: -100, duration: 500 }}
							>
								<PaintComponent
									{paint}
									enableDialog
									onmouseenter={() => (paintMouseOver = true)}
									onmouseleave={() => (paintMouseOver = false)}
								>
									<h2>
										{#if paintMouseOver}
											{username}
										{:else}
											{paint.name}
										{/if}
									</h2>
								</PaintComponent>
							</div>
						{/if}
					{/each}
				</div>
			</div>
			<div class="badge-preview hide-on-mobile">
				<div class="badges">
					{#each badges as badge}
						<BadgeComponent enableDialog size={48} {badge} />
					{/each}
				</div>
			</div>
			<div class="dismiss">
				<Button onclick={dismiss}>
					{#snippet icon()}
						<X />
					{/snippet}
				</Button>
			</div>
		</div>
	</div>
{/if}

<style lang="scss">
	@keyframes backgroundScroll {
		0% {
			background-position: 0 0;
		}
		100% {
			background-position: 0 -40rem;
		}
	}

	.pickems-header-layout {
		padding: 1.25rem;
		padding-bottom: 0rem;

		.pickems-header {
			position: relative;
			border: solid 1px hsla(0deg, 0%, 100%, 20%);
			border-radius: 0.5rem;
			padding: 2rem;

			display: grid;
			gap: 1rem;
			grid-template-columns: 2fr 1fr 2fr;

			overflow: clip;
			&::after {
				content: "";
				position: absolute;
				inset: 0 0 0 0;
				background: radial-gradient(10rem circle at 10rem 0, purple, transparent),
					radial-gradient(10rem circle at 10rem 20rem, pink, transparent),
					radial-gradient(10rem circle at 10rem 40rem, purple, transparent),
					radial-gradient(10rem circle at 30rem 10rem, lightblue, transparent),
					radial-gradient(10rem circle at 30rem 30rem, blue, transparent),
					radial-gradient(10rem circle at 50rem 0, yellow, transparent),
					radial-gradient(10rem circle at 50rem 20rem, red, transparent),
					radial-gradient(10rem circle at 50rem 40rem, yellow, transparent),
					radial-gradient(10rem circle at 70rem 10rem, blue, transparent),
					radial-gradient(10rem circle at 70rem 30rem, orange, transparent);
				background-size: 80rem 40rem;
				filter: blur(4rem);

				animation: backgroundScroll 20s linear infinite;
				z-index: 0;
			}
			> * {
				z-index: 1;
			}

			.purchase-info {
				display: flex;
				flex-direction: column;
				gap: 0.5rem;

				p {
					color: silver;
					font-size: small;
				}

				.buttons {
					padding-top: 1rem;
					display: flex;
					gap: 0.5rem;
				}
			}

			.paint-preview {
				display: flex;
				justify-content: center;
				align-items: center;

				.paint {
					position: relative;
					height: 50%;
					width: 12rem;
					display: flex;
					justify-content: center;
					padding: 1rem 2rem;
					background: hsla(0deg, 0%, 40%, 20%);
					backdrop-filter: blur(2rem);
					border-radius: 0.5rem;
					overflow: clip;

					.paint-inner {
						position: absolute;
					}
				}
			}

			.badge-preview {
				display: flex;
				justify-content: end;
				align-items: center;

				.badges {
					display: flex;
					gap: 1rem;
					padding: 1rem;
					background: hsla(0deg, 0%, 40%, 20%);
					backdrop-filter: blur(2rem);
					border-radius: 0.5rem;
				}
			}
		}

		.dismiss {
			position: absolute;
			top: 0.5rem;
			right: 0.5rem;
		}
	}

	@media screen and (max-width: 960px) {
		.pickems-header-layout {
			padding: 0.5rem;
			padding-bottom: 0rem;
			.pickems-header {
				grid-template-columns: 1fr;

				.badge-preview {
					justify-content: center;
				}
			}
		}
	}
</style>
