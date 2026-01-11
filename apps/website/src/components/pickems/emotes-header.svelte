<script lang="ts">
	import { ArrowRight, Ticket, X } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { type Paint, type Badge } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import { queryPickemsCosmetics } from "$/lib/pickems";
	import PickemsBadges from "./pickems-badges.svelte";
	import PickemsPaints from "./pickems-paints.svelte";
	import { t } from "svelte-i18n";

	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);

	let dismissed = $state(window.localStorage.getItem("pickems-dismissed") ?? "false");
	function dismiss() {
		dismissed = "true";
		window.localStorage.setItem("pickems-dismissed", "true");
	}

	let badges = $state<Badge[]>([]);
	let paints = $state<Paint[]>([]);

	$effect(() => {
		const emotesDirectoryElement = document.querySelector(
			".emotes-side-bar-layout, .emotes-side-bar-layout-full",
		);
		if (emotesDirectoryElement) {
			emotesDirectoryElement.classList.toggle("emotes-side-bar-layout-full", dismissed === "true");
			emotesDirectoryElement.classList.toggle("emotes-side-bar-layout", dismissed !== "true");
		}
		queryPickemsCosmetics().then((cosmetics) => {
			badges = cosmetics.badges;
			paints = cosmetics.paints;
		});
	});
</script>

{#if dismissed !== "true"}
	<div class="pickems-header-layout">
		<div class="pickems-header">
			<div class="purchase-info">
				<div class="text">
					<p class="pre-title">
						{$t("pages.store.events.cs2.tournament")} &#x2022 {$t("pages.landing.hero.cs2.date")}
					</p>
					<h2 class="title">{$t("pages.landing.hero.cs2.header")}</h2>
				</div>
				<div class="buttons">
					{#if hasPass}
						<Button primary href="/store/pickems">
							{#snippet iconRight()}
								<ArrowRight />
							{/snippet}
							{$t("pages.store.events.cs2.pickems.place")}
						</Button>
					{:else}
						<Button href="/store/pickems" primary>
							{#snippet iconRight()}
								<Ticket />
							{/snippet}
							{$t("pages.store.events.cs2.pickems.purchase_pass")}
						</Button>
					{/if}
				</div>
			</div>
			<div class="paint-preview hide-on-mobile">
				<PickemsPaints {paints} />
			</div>
			<div class="badge-preview hide-on-mobile">
				<PickemsBadges {badges} />
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
		padding: 1.25rem 1.25rem 0;

		.pickems-header {
			position: relative;
			border: solid 1px hsla(0deg, 0%, 100%, 20%);
			border-radius: 0.5rem;
			padding: 1rem 3rem 1rem 1rem;

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
				gap: 0.5rem;
				align-items: center;

				p {
					color: silver;
					font-size: small;
				}

				.buttons {
					padding-left: 1rem;
				}
			}

			.paint-preview {
				display: flex;
				justify-content: center;
				align-items: center;
			}

			.badge-preview {
				display: flex;
				justify-content: end;
				align-items: center;
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
			padding: 0.5rem 0.5rem 0;
			.pickems-header {
				grid-template-columns: 1fr;

				.purchase-info {
					flex-direction: column;
					justify-content: left;
				}

				.badge-preview {
					justify-content: center;
				}
			}
		}
	}
</style>
