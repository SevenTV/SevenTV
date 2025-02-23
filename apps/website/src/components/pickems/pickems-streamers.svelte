<script lang="ts">
	import { ArrowSquareOut, ArrowUpRight, Ticket } from "phosphor-svelte";
	import xQcProfile from "$assets/xqc.png?url";
	import stableRonaldoProfile from "$assets/stableronaldo.png?url";
	import aikoBlissProfile from "$assets/aikobliss.png?url";
	import dorozeaProfile from "$assets/dorozea.png?url";
	import erobb221Profile from "$assets/erobb221.png?url";
	import donaCSProfile from "$assets/donacs.png?url";
	import jasonTheWeenProfile from "$assets/jasontheween.png?url";
	import renyanProfile from "$assets/renyan.png?url";
	import omieProfile from "$assets/omie.png?url";
	import arrowCSProfile from "$assets/arrowcs.png?url";
	import anomalyProfile from "$assets/anomaly.png?url";
	import dimaWallhacksProfile from "$assets/dima_wallhacks.png?url";
	import ohnePixelProfile from "$assets/ohnepixel.png?url";
	import furiProfile from "$assets/furi.png?url";
	import kaiCanetProfile from "$assets/kaicanet.png?url";
	import dizzyProfile from "$assets/dizzy.png?url";
	import Button from "../input/button.svelte";

	let { hasPass }: { hasPass: Boolean } = $props();

	let streamers = [
		{
			name: "xQc",
			profile: xQcProfile,
		},
		{
			name: "StableRonaldo",
			profile: stableRonaldoProfile,
		},
		{
			name: "AikoBliss",
			profile: aikoBlissProfile,
		},
		{
			name: "Dorozea",
			profile: dorozeaProfile,
		},
		{
			name: "Erobb221",
			profile: erobb221Profile,
		},
		{
			name: "DonaCS",
			profile: donaCSProfile,
		},
		{
			name: "JasonTheWeen",
			profile: jasonTheWeenProfile,
		},
		{
			name: "Renyan",
			profile: renyanProfile,
		},
		{
			name: "omie",
			profile: omieProfile,
		},
		{
			name: "ArrowCS",
			profile: arrowCSProfile,
		},
		{
			name: "anomaly",
			profile: anomalyProfile,
		},
		{
			name: "dima_wallhacks",
			profile: dimaWallhacksProfile,
		},
		{
			name: "OhnePixel",
			profile: ohnePixelProfile,
		},
		{
			name: "Furi",
			profile: furiProfile,
		},
		{
			name: "KaiCenat",
			profile: kaiCanetProfile,
		},
		{
			name: "Dizzy",
			profile: dizzyProfile,
		},
	];
</script>

<div class="container">
	<div class="left-panel">
		<h1>Meet the Streamers.</h1>
		<hr class="hrDialog" />
		<p>
			16 Streamers each build a team from their community to come head to head with each other in a
			Single Elimination Tournament.
		</p>
		<p>
			Watch the main event on <a href="https://twitch.tv/7tvapp" aria-label="7tvapp Twitch Channel"
				>7TV Twitch Channel <ArrowSquareOut style="vertical-align: text-top;" /></a
			>, or through your favourite streamer's POV.
		</p>
		<p>Want to earn cosmetics for predicting the outcome? Click below!</p>
		<div class="purchase-div">
			{#if !hasPass}
				<Button
					style="width: 100%;
				padding: 0.75rem 1.5rem;
				border-radius: 8px;
				background: rgb(89 5 199);
    			box-shadow: rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset;
				font-family: Inter;
				color: white;
				border: none;
				border-radius: 0.25rem;
				cursor: pointer;"
					href="https://pickems.tv"
				>
					Predict Winners
					<Ticket style="margin-left: auto; margin-right: 0;" />
				</Button>
			{:else}
				<Button
					style="width: 100%;
				padding: 0.75rem 1.5rem;
				border-radius: 8px;
				background: rgb(89 5 199);
    			box-shadow: rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset;
				font-family: Inter;
				color: white;
				border: none;
				border-radius: 0.25rem;
				cursor: pointer;"
					href="#PickemsPurchaseButton"
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
				<div class="streamer">
					<img
						class="streamer-pfp"
						alt={`${streamer.name}'s profile picture`}
						src={streamer.profile}
					/>
					<div class="info">
						<h3>{streamer.name}</h3>
						<a href={`https://twitch.tv/${streamer.name}`} target="_blank">
							<u> VIEW TWITCH </u>
							<ArrowUpRight />
						</a>
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>

<style lang="scss">
	.hrDialog {
		background: rgba(255, 255, 255, 0.07);
		width: 100%;
		margin: 0;
	}
	.container {
		display: flex;
		width: 100%;
		background-color: black;
	}

	.left-panel {
		gap: 1rem;
		border-radius: 8px;
		border-right: 1px solid rgba(255, 255, 255, 0.15);
		width: 25%;
		padding: 2rem;
		background: linear-gradient(
			180deg,
			rgba(175, 175, 175, 0.16) 0%,
			rgba(113, 113, 113, 0.16) 100%
		);
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
			margin-bottom: 1rem;
		}

		p {
			margin-bottom: 2rem;
			color: rgba(255, 255, 255, 0.5);

			font-family: Inter;
		}
		.purchase-div {
			margin-top: auto;
			width: 100%;
		}
	}

	.right-panel {
		border-radius: 12px;
		border: 1px solid rgba(255, 255, 255, 0.15);
		width: 75%;
		padding: 2rem;
		overflow-y: auto;
	}

	.streamer-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 1rem;

		.streamer {
			display: flex;
			align-items: center;
			padding: 0.5rem;
			background-color: #000000;
			border-radius: 0.5rem;
			box-shadow: 0 0.5rem 1rem rgba(0, 0, 0, 0.1);

			img {
				border-radius: 0; /* Make the image square */
				height: 4rem; /* Increase the size of the image */
				width: 4rem;
				margin-right: 0.5rem; /* Add margin to the right */
			}

			.info {
				text-align: left;

				h3 {
					font-size: 1rem; /* Make the username smaller */
				}

				a {
					display: flex;
					align-items: center;
					color: grey;
					margin-top: 0.5rem;
					font-size: 0.875rem; /* Make the view twitch text smaller */

					:global(svg) {
						margin-left: 0.5rem;
					}
				}
			}
		}
	}
</style>
