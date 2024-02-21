<script lang="ts">
	import {
		faBadge,
		faBrush,
		faSmilePlus,
		faTicket,
		faUserCircle,
	} from "@fortawesome/pro-regular-svg-icons";
	import Fa from "svelte-fa";
</script>

<section>
	<h2>Subscription Benefits</h2>
	<div class="benefits">
		<div class="benefit">
			<div class="gradient g1">
				<Fa icon={faUserCircle} size="3x" />
			</div>
			<h3>Animated Profile Picture</h3>
		</div>
		<div class="benefit">
			<div class="gradient g2">
				<Fa icon={faSmilePlus} size="3x" />
			</div>
			<h3>Personal Emotes</h3>
		</div>
		<div class="benefit">
			<div class="gradient g3">
				<Fa icon={faBrush} size="3x" />
			</div>
			<h3>Nametag Paints</h3>
		</div>
		<div class="benefit">
			<div class="gradient g5">
				<Fa icon={faBadge} size="3x" />
			</div>
			<h3>Subscriber Badge</h3>
		</div>
		<div class="benefit">
			<div class="gradient g6">
				<Fa icon={faTicket} size="3x" />
			</div>
			<h3>Raffle Ticket</h3>
		</div>
	</div>
</section>

<style lang="scss">
	section {
		flex-basis: 0;
		flex-grow: 1;
		width: 100%;
	}

    h2 {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 1rem;
	}

	h3 {
		font-size: 0.75rem;
		font-weight: 500;
	}

	.benefits {
		display: flex;
		gap: 1rem;

        overflow-x: auto;
		width: 100%;
	}

	.benefit {
		flex-basis: 0;
		flex-grow: 1;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		display: flex;
		flex-direction: column;
		text-align: center;

        cursor: pointer;

        transition: background-color 0.2s;

        &:hover, &:focus-visible {
            background-color: var(--bg-light);
        }

		.gradient {
			height: 8.5rem;

			border-radius: 0.5rem;
			display: flex;
			justify-content: center;
			align-items: center;

			&.g1 {
				background: radial-gradient(
						150% 180% at 50% 180%,
						#ff922f 0%,
						#ffb800 33%,
						#43006d 64%,
						#0f0f0f 80%
					),
					var(--secondary);
			}

			&.g2 {
				background: radial-gradient(
						150% 180% at 50% 180%,
						#fff 0%,
						#ff3535 33%,
						#43006d 64%,
						#0f0f0f 80%
					),
					var(--secondary);
			}

			&.g3 {
				background: radial-gradient(
						150% 180% at 50% 180%,
						#ff922f 0%,
						#ff11bc 33%,
						#43006d 64%,
						#0f0f0f 80%
					),
					var(--secondary);
			}

			&.g5 {
				background: radial-gradient(
						150% 180% at 50% 180%,
						#ff922f 0%,
						#11f1ff 33%,
						#43006d 64%,
						#0f0f0f 80%
					),
					var(--secondary);
			}

			&.g6 {
				background: radial-gradient(
						150% 180% at 50% 180%,
						#ff922f 0%,
						#11ff54 33%,
						#43006d 64%,
						#0f0f0f 80%
					),
					var(--secondary);
			}
		}
	}
</style>
