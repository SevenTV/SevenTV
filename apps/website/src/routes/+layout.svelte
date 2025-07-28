<svelte:options runes={true} />

<script lang="ts">
	import "$/styles/fonts.scss";
	import "$/styles/variables.scss";
	import "$/styles/global.scss";
	import "$/lib/i18n";
	import "$/lib/emoteSets";
	import TopNav from "$/components/nav/top-nav.svelte";
	import {
		showMobileMenu,
		showConstructionBar,
		signInDialogMode,
		signInDialogPayload,
		uploadDialogMode,
		defaultEmoteSetDialogMode,
		showEasterBar,
		showSummerGift,
	} from "$/lib/layout";
	import summerBeachIcon from "$assets/summer-beach.webp?url";
	import Menu from "$/components/nav/menu.svelte";
	import { beforeNavigate } from "$app/navigation";
	import { IconContext, Warning, X, CaretDown } from "phosphor-svelte";
	import PaintComponent from "$/components/paint.svelte";
	import BadgeComponent from "$/components/badge.svelte";
	import UploadDialog from "$/components/dialogs/upload-dialog.svelte";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import DefaultEmoteSetDialog from "$/components/dialogs/default-emote-set-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import ErrorDialog from "$/components/dialogs/error-dialog.svelte";
	import { currentError, errorDialogMode } from "$/lib/error";
	import { PUBLIC_DISCORD_LINK, PUBLIC_OLD_WEBSITE_LINK } from "$env/static/public";
	import type { Badge, Maybe, Paint } from "$/gql/graphql";
	import Button from "../components/input/button.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	const summerBadgeID = "01JZX7H1H6QERDGF8QRA2ZAKPW";
	const summerPaintID = "01JZ3NFYMT7NBFMEP0HKJTV5W8";
	let summerPaintData = $state<Maybe<Paint>[]>([]);
	let summerBadgeData = $state<Maybe<Badge>[]>([]);
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

	let { children }: { children: Snippet } = $props();

	let expanded = $state(false);
	function toggleExpand() {
		expanded = !expanded;
		const bar = document.querySelector(".alert-bar") as HTMLElement;
		if (bar) {
			bar.style.transitionDelay = expanded ? "100ms" : "0ms";
			bar.style.padding = expanded ? "1.8rem" : "0.5rem";
			const extra = bar.querySelector(".extra-bar-content") as HTMLElement;
			if (extra) {
				extra.style.opacity = expanded ? "1" : "0";
				extra.style.visibility = expanded ? "visible" : "hidden";
				extra.style.maxHeight = expanded ? "600px" : "0";
				extra.style.transitionDelay = expanded ? "100ms" : "0ms";
			}
		}
	}

	beforeNavigate((nav) => {
		// Hide menu on navigate
		nav.complete.then(() => {
			$showMobileMenu = false;
		});
	});

	$effect(() => {
		document.body.classList.toggle("construction-bar", $showEasterBar);
		document.body.classList.toggle("summer-gift", $showSummerGift);

		(async () => {
			const cosmetics = await getAllCosmetics();
			let paint = cosmetics.paints.find((paint: { id: string }) => paint.id === summerPaintID);
			let badge = cosmetics.badges.find((badge: { id: string }) => badge.id === summerBadgeID);
			summerPaintData = paint ? [paint as Maybe<Paint>] : [];
			summerBadgeData = badge ? [badge as Maybe<Badge>] : [];
			console.log("Summer Paint Data:", summerPaintData);
			console.log("Summer Badge Data:", summerBadgeData);
		})();
	});
</script>

<IconContext values={{ size: 1.2 * 16, weight: "bold", style: "flex-shrink: 0" }}>
	<header>
		<a href="#main" class="skip-to-main">{$t("common.skip_to_content")}</a>
		<TopNav />
		{#if $showConstructionBar && false}
			<div class="alert-bar">
				<Warning />
				<span>Under construction</span>
				<span class="small">
					Give us feedback on the new website in the <a href={PUBLIC_DISCORD_LINK}>Discord</a>.
					Click
					<a href={PUBLIC_OLD_WEBSITE_LINK}>here</a> to return to the old website.
				</span>
				<Button onclick={() => ($showConstructionBar = false)}>
					{#snippet icon()}
						<X />
					{/snippet}
				</Button>
			</div>
		{/if}
		{#if $showEasterBar}
			<div class="alert-bar">
				<span>
					Happy Easter!
					<a href="/store"> Gift 4 times to receive a limited edition Easter Badge and Paint! </a>
				</span>
				<Button onclick={() => ($showEasterBar = false)}>
					{#snippet icon()}
						<X />
					{/snippet}
				</Button>
			</div>
		{/if}
		{#if $showSummerGift}
			<div class="alert-bar">
				<span class="main-text" style="display: flex; align-items: center; gap: 0.5rem;">
					<Button
						onclick={toggleExpand}
						aria-expanded={expanded}
						aria-label={expanded ? "Collapse details" : "Expand details"}
						style="position: static; right: 70%; bottom: 0.5rem; z-index: 2;"
					>
						{#snippet icon()}
							<CaretDown
								style={`transform: rotate(${expanded ? 180 : 0}deg); transition: transform 0.2s;`}
							/>
						{/snippet}
					</Button>
					<img src={summerBeachIcon} alt="Summer Gift Icon" style="width: 3rem; height: 3rem;" />
					The Summer Gift Event is Here!
					<a href="/store"> Gift 4 times to receive a limited edition Summer Badge and Paint! </a>
					<Button onclick={() => ($showSummerGift = false)}>
						{#snippet icon()}
							<X />
						{/snippet}
					</Button>
				</span>
				<div class="extra-bar-content">
					<br />
					<div style="display: flex; gap: 5.5rem;">
						{#if summerBadgeData.length > 0 && summerBadgeData[0]}
							<BadgeComponent badge={summerBadgeData[0]} size={2 * 32} />
						{/if}
						{#if summerPaintData.length > 0 && summerPaintData[0]}
							<div
								style="display: flex; justify-content: center; align-items: center; width: 100%;"
							>
								<PaintComponent paint={summerPaintData[0]}>Summer Gifting</PaintComponent>
							</div>
						{/if}
					</div>
					<br />
					<span class="more-info">
						You can gift 4 7TV subs to friends and earn the rewards above. Rewards are only
						available during the event!
					</span>
					<br />
					<Button href="/store" onclick={toggleExpand} primary>Interested? Click here</Button>
				</div>
			</div>
		{/if}
	</header>

	<UploadDialog bind:mode={$uploadDialogMode} />
	<SignInDialog bind:mode={$signInDialogMode} bind:return_payload={$signInDialogPayload} />
	<DefaultEmoteSetDialog bind:mode={$defaultEmoteSetDialogMode} />
	<ErrorDialog bind:mode={$errorDialogMode} error={$currentError} />
	<main id="main">
		{#if $showMobileMenu}
			<Menu onCloseRequest={() => ($showMobileMenu = false)} />
		{:else}
			{@render children()}
		{/if}
	</main>
</IconContext>

<style lang="scss">
	:global(body) {
		max-height: 100vh; /* For browsers that don't support svh */
		max-height: 100svh;
		min-height: 100vh; /* For browsers that don't support svh */
		min-height: 100svh;

		display: grid;
		grid-template-rows: auto 1fr;
	}

	:global(body.construction-bar) {
		grid-template-rows: auto auto 1fr;
	}

	// header {
	// 	display: contents;
	// }

	.alert-bar {
		background-color: var(--bg-light);
		padding: 0.5rem;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--text);
		font-weight: 600;
		position: relative;
		transition: padding 0.3s ease;

		.main-text {
			text-align: center;
		}

		.extra-bar-content {
			opacity: 0;
			visibility: hidden;
			max-height: 0;
			overflow: hidden;
			transition: all 0.3s ease;
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: 0.5rem;
			text-align: center;
		}
	}

	main {
		overflow: auto;
	}

	.skip-to-main {
		position: absolute;
		color: var(--primary);
		opacity: 0;
		pointer-events: none;

		&:focus-visible {
			opacity: 1;
			pointer-events: unset;
		}
	}
</style>
