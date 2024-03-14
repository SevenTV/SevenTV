<script lang="ts">
	import { DotsThreeVertical } from "phosphor-svelte";
	import StoreSection from "./store-section.svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";

	export let percentage: number = 40;
</script>

<StoreSection>
	<div class="container">
		<div class="progress-circle">
			<svg width="128" height="128" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
				<!-- 64 - 8 = 56 -->
				<circle id="track" cx="64" cy="64" r="56" fill="none"></circle>
				<!-- 2pi * 56 = 356 -->
				<circle
					id="progress"
					cx="64"
					cy="64"
					r="56"
					fill="none"
					stroke-dasharray="356"
					style="--offset: {(1 - percentage / 100) * 356}"
				></circle>

				<defs>
					<linearGradient id="gradient">
						<stop offset="0%" stop-color="#5d25fe"></stop>
						<stop offset="100%" stop-color="#ff36f7"></stop>
					</linearGradient>
				</defs>
			</svg>
			<span>{percentage}%</span>
		</div>
		<div class="info">
			<div class="header">
				<h2>{$t("pages.store.subscription.badge_progress")}</h2>
				<Button secondary>
					<DotsThreeVertical slot="icon" />
				</Button>
			</div>
			<div class="badges">
				<div class="badge">
					<div class="placeholder"></div>
					<span>1 Year</span>
				</div>
				<div class="bar-container">
					<span class="countdown">23 days left</span>
					<div class="bar"></div>
				</div>
				<div class="badge">
					<div class="placeholder"></div>
					<span>1.5 Years</span>
				</div>
			</div>
		</div>
	</div>
</StoreSection>

<style lang="scss">
	.container {
		display: flex;
		gap: 1.25rem;
	}

	// https://stackoverflow.com/a/69183742/10772729
	.progress-circle {
		width: 8rem;
		height: 8rem;

		font-size: 1.25rem;
		font-weight: 700;

		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;

		& > svg {
			position: absolute;
			transform: rotate(-90deg);
		}

		#progress {
			stroke: url(#gradient);
			stroke-width: 1rem;
			stroke-linecap: round;

			animation: circle-progress 0.5s forwards;
		}

		#track {
			stroke: var(--secondary);
			stroke-width: 1rem;
		}
	}

	@keyframes circle-progress {
		from {
			stroke-dashoffset: 356;
		}

		to {
			stroke-dashoffset: var(--offset);
		}
	}

	.info {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		justify-content: space-between;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;

		h2 {
			font-size: 1rem;
			font-weight: 600;
		}
	}

	.badges {
		display: flex;
		gap: 0.5rem;

		color: var(--text-light);
		font-size: 0.75rem;
		font-weight: 600;

		.badge {
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: 0.5rem;

			& > .placeholder {
				width: 2.25rem;
				height: 2.25rem;
				background-color: var(--secondary);
				border-radius: 0.5rem;
			}
		}

		.bar-container {
			flex-grow: 1;
			height: 2.25rem;
			position: relative;

			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;

			.countdown {
				position: absolute;
				top: -0.5rem;
				white-space: nowrap;
			}

			.bar {
				align-self: stretch;

				height: 0.25rem;
				background-color: var(--secondary);
				border-radius: 0.125rem;
			}
		}
	}
</style>
