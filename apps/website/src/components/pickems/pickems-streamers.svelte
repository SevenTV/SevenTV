<script lang="ts">
	import { ArrowSquareOut, Ticket } from "phosphor-svelte";
	// import xQcProfile from "$assets/xqc.png?url";
	// import stableRonaldoProfile from "$assets/stableronaldo.png?url";
	// import aikoBlissProfile from "$assets/aikobliss.png?url";
	// import dorozeaProfile from "$assets/dorozea.png?url";
	// import erobb221Profile from "$assets/erobb221.png?url";
	// import donaCSProfile from "$assets/donacs.png?url";
	// import jasonTheWeenProfile from "$assets/jasontheween.png?url";
	// import renyanProfile from "$assets/renyan.png?url";
	// import omieProfile from "$assets/omie.png?url";
	// import arrowCSProfile from "$assets/arrowcs.png?url";
	// import dimaWallhacksProfile from "$assets/dima_wallhacks.png?url";
	// import ohnePixelProfile from "$assets/ohnepixel.png?url";
	// import furiProfile from "$assets/furi.png?url";
	// import neokProfile from "$assets/neok.png?url";
	// import dizzyProfile from "$assets/dizzy.png?url";
	// import StewieProfile from "$assets/stewie2k.png?url";
	// import duwapProfile from "$assets/duwap.png?url";
	// import trilluxeProfile from "$assets/trilluxe.png?url";
	// import trainwreckProfile from "$assets/trainwreckstv.png?url";
	// import propernatorProfile from "$assets/propernator.png?url";
	// import andersProfile from "$assets/anders.png?url";
	// import moetvProfile from "$assets/m0e_tv.png?url";
	// import moxyProfile from "$assets/m0xyy.jpg?url";
	// import austinProfile from "$assets/austincs_.png?url";
	// import hyzacsProfile from "$assets/hyzacs.png?url";
	// import jojodjProfile from "$assets/jojodj.png?url";
	// import tylert69Profile from "$assets/tylert69_.png?url";
	// import segalsonProfile from "$assets/segalson.png?url";
	// import loagurtProfile from "$assets/loagurt.png?url";
	// import coachmaryyProfile from "$assets/coachmaryy.png?url";
	// import pol0Profile from "$assets/pol0.png?url";
	// import trucklover86Profile from "$assets/trucklover86.png?url";
	// import dathrixProfile from "$assets/dathrix.png?url";
	// import dementorcsgoProfile from "$assets/dementorcsgo.png?url";
	// import destroybladerProfile from "$assets/destroyblader.png?url";

	import Button from "../input/button.svelte";

	let { hasPass = $bindable() }: { hasPass: boolean } = $props();

	let streamers: { name: string; profile: string }[] = [
		// {
		// 	name: "xQc",
		// 	profile: xQcProfile,
		// },
		// {
		// 	name: "StableRonaldo",
		// 	profile: stableRonaldoProfile,
		// },
		// {
		// 	name: "AikoBliss",
		// 	profile: aikoBlissProfile,
		// },
		// {
		// 	name: "Dorozea",
		// 	profile: dorozeaProfile,
		// },
		// {
		// 	name: "Erobb221",
		// 	profile: erobb221Profile,
		// },
		// {
		// 	name: "DonaCS",
		// 	profile: donaCSProfile,
		// },
		// {
		// 	name: "JasonTheWeen",
		// 	profile: jasonTheWeenProfile,
		// },
		// {
		// 	name: "Renyan",
		// 	profile: renyanProfile,
		// },
		// {
		// 	name: "omie",
		// 	profile: omieProfile,
		// },
		// {
		// 	name: "ArrowCS",
		// 	profile: arrowCSProfile,
		// },
		// {
		// 	name: "dima_wallhacks",
		// 	profile: dimaWallhacksProfile,
		// },
		// {
		// 	name: "OhnePixel",
		// 	profile: ohnePixelProfile,
		// },
		// {
		// 	name: "Furi",
		// 	profile: furiProfile,
		// },
		// {
		// 	name: "neok",
		// 	profile: neokProfile,
		// },
		// {
		// 	name: "Dizzy",
		// 	profile: dizzyProfile,
		// },
		// {
		// 	name: "Stewie2K",
		// 	profile: StewieProfile,
		// },
		// {
		// 	name: "duwap",
		// 	profile: duwapProfile,
		// },
		// {
		// 	name: "TrilluXe",
		// 	profile: trilluxeProfile,
		// },
		// {
		// 	name: "trainwreckstv",
		// 	profile: trainwreckProfile,
		// },
		// {
		// 	name: "propernator",
		// 	profile: propernatorProfile,
		// },
		// {
		// 	name: "anders",
		// 	profile: andersProfile,
		// },
		// {
		// 	name: "m0e_tv",
		// 	profile: moetvProfile,
		// },
		// {
		// 	name: "m0xyy",
		// 	profile: moxyProfile,
		// },
		// {
		// 	name: "austincs_",
		// 	profile: austinProfile,
		// },
		// {
		// 	name: "hyzaCS",
		// 	profile: hyzacsProfile,
		// },
		// {
		// 	name: "jojodj",
		// 	profile: jojodjProfile,
		// },
		// {
		// 	name: "tylert69_",
		// 	profile: tylert69Profile,
		// },
		// {
		// 	name: "segalson",
		// 	profile: segalsonProfile,
		// },
		// {
		// 	name: "loagurt",
		// 	profile: loagurtProfile,
		// },
		// {
		// 	name: "coachmaryy",
		// 	profile: coachmaryyProfile,
		// },
		// {
		// 	name: "pol0",
		// 	profile: pol0Profile,
		// },
		// {
		// 	name: "trucklover86",
		// 	profile: trucklover86Profile,
		// },
		// {
		// 	name: "dathrix",
		// 	profile: dathrixProfile,
		// },
		// {
		// 	name: "dementorcsgo",
		// 	profile: dementorcsgoProfile,
		// },
		// {
		// 	name: "destroyblader",
		// 	profile: destroybladerProfile,
		// },
	];
</script>

<div class="container">
	<div class="left-panel">
		<h1 class="alt_head">Meet the Streamers</h1>
		<hr class="hrDialog" />
		<p>
			A prominent line-up of Twitch Streamers build a team from their communities who go
			head-to-head in a CS-Style Single Elimination Tournament.
		</p>
		<p>
			Watch the main event on the <a
				href="https://twitch.tv/7tvapp"
				aria-label="7tvapp Twitch Channel"
				>7TV Twitch Channel <ArrowSquareOut style="vertical-align: text-top;" /></a
			>, or through your favourite streamer's POV.
		</p>
		<p>Want to earn cosmetics for predicting the outcome? Click below!</p>
		<div class="purchase-div">
			{#if hasPass}
				<Button
					style="width: 100%;
				padding: 0.75rem 1.5rem;
				background: rgb(89 5 199);
    			box-shadow: rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset;
				font-family: Inter;
				color: white;
				border: none;
				border-radius: 0.25rem;
				cursor: pointer;"
					href="https://app.pickems.tv"
				>
					Predict Winners
					<ArrowSquareOut style="margin-left: auto; margin-right: 0;" />
				</Button>
			{:else}
				<Button
					style="width: 100%;
				padding: 0.75rem 1.5rem;
				background: rgb(89 5 199);
    			box-shadow: rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset;
				font-family: Inter;
				color: white;
				border: none;
				border-radius: 0.25rem;
				cursor: pointer;"
					href="#PickemsPricing"
				>
					Purchase Pass
					<Ticket style="margin-left: auto; margin-right: 0;" />
				</Button>
			{/if}
		</div>
	</div>
	<div class="right-panel">
		<div class="streamer-grid">
			{#each streamers as streamer}
				<a href={`https://twitch.tv/${streamer.name}`} target="_blank" class="streamer-box">
					<div class="streamer">
						<img
							class="streamer-pfp"
							alt={`${streamer.name}'s profile picture`}
							src={streamer.profile}
						/>
						<div class="info">
							<h3>{streamer.name}</h3>
							<span class="streamer-link">
								<u> VIEW TWITCH </u>
							</span>
						</div>
					</div>
				</a>
			{/each}
		</div>
	</div>
</div>

<style lang="scss">
	.streamer-pfp {
		border-radius: 4px;
		margin-right: 0.75rem;
	}
	.hrDialog {
		background: rgba(255, 255, 255, 0.07);
		width: 100%;
		margin: 0;
	}
	.container {
		display: flex;
		width: 100%;
		background-color: transparent;
		flex-direction: row;
	}

	.streamer-box {
		text-decoration: none;
	}

	.left-panel {
		background: linear-gradient(180deg, rgba(175, 175, 175, 0.2), rgba(175, 175, 175, 0.14));
		border-right: none;
		border-radius: 12px 0 0 12px;
		gap: 1rem;
		width: 25%;
		padding: 1.25rem 2.5rem;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		justify-content: flex-start;

		h1 {
			margin-bottom: 0.5rem;
		}

		hr {
			width: 100%;
			border: 0;
			border-top: 1px solid rgba(255, 255, 255, 0.15);
			margin-bottom: 0.5rem;
		}

		p {
			margin-bottom: 2rem;
			color: rgba(255, 255, 255, 0.7);
			font-family: Inter, serif;
		}
		.purchase-div {
			margin-top: auto;
			width: 100%;
		}
	}

	.right-panel {
		border: 1px solid rgba(255, 255, 255, 0.15);
		width: 75%;
		padding: 1.25rem 2.5rem;
		overflow: hidden;
		border-radius: 0 12px 12px 0;
		justify-content: center;
		align-items: center;
		background-color: black;
	}

	.streamer-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2rem;

		.streamer {
			display: flex;
			align-items: center;
			padding: 0.5rem;
			background-color: #191919;
			border-radius: 0.5rem;
			box-shadow: 0 0.5rem 1rem rgba(0, 0, 0, 0.1);
			color: #999999;

			img {
				border-radius: 0.3rem;
				height: 4rem;
				width: 4rem;
				margin-right: 0.5rem;
			}

			.streamer-link {
				font-size: 0.75rem;
			}

			.info {
				text-align: left;

				h3 {
					font-size: 1rem;
				}

				a {
					display: flex;
					align-items: center;
					color: grey;
					margin-top: 0.5rem;
					font-size: 0.875rem;

					:global(svg) {
						margin-left: 0.5rem;
					}
				}
			}
		}
		.streamer:hover {
			transition: ease-in-out 0.3s;
			background-color: rgba(255, 255, 255, 0.15);
			color: white;
		}
	}

	.alt_head {
		font-size: 1.5rem;
		font-weight: 600;
	}

	@media screen and (max-width: 1600px) {
		.streamer-grid {
			grid-template-columns: repeat(3, 1fr) !important;
		}
	}

	@media screen and (max-width: 1400px) {
		.streamer-grid {
			grid-template-columns: repeat(2, 1fr) !important;
		}
	}

	@media screen and (max-width: 1200px) {
		.container {
			flex-direction: column !important;
		}
		.left-panel {
			width: 100% !important;
			border-radius: 12px 12px 12px 12px !important;
		}
		.right-panel {
			width: 100% !important;
			border-radius: 12px 12px 12px 12px !important;
			margin-top: 1rem !important;
		}

		.streamer-grid {
			grid-template-columns: repeat(2, 1fr) !important;
			gap: 1rem !important;
		}
	}

	@media screen and (max-width: 500px) {
		.container {
			flex-direction: column !important;
		}
		.left-panel {
			width: 100% !important;
			border-radius: 12px 12px 12px 12px !important;
			text-align: center;
		}
		.right-panel {
			width: 100% !important;
			border-radius: 12px 12px 12px 12px !important;
			margin-top: 1rem !important;
		}
		.streamer-grid {
			grid-template-columns: repeat(1, 1fr) !important;
			gap: 1rem !important;
		}
		.alt_head {
			text-align: center;
			width: 100%;
			font-size: 1.3rem;
		}
	}
</style>
