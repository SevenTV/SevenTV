<script lang="ts">
	import moment from "moment";
	import { Clock, Coin, Gift, Hourglass, Sparkle } from "phosphor-svelte";
</script>

<div class="sub-grid" {...$$restProps}>
	<span class="key">Plan</span>
	<span class="value">
		<Sparkle />
		<span>Free</span>
	</span>

	<span class="key">Gifted By</span>
	<span class="value">
		<Gift />
		<span>Forsen</span>
	</span>

	<span class="key">Sub Ends</span>
	<span class="value">
		<Hourglass />
		<span>{moment().format("ll")}</span>
	</span>

	<span class="key">Total</span>
	<span class="value">
		<Clock />
		<span>{moment.duration(317, "days").humanize({ d: Infinity })}</span>
	</span>

	<span class="key">Sub Credits</span>
	<span class="value">
		<Coin />
		<span>3</span>
	</span>
</div>

<style lang="scss">
	.sub-grid {
		padding: 0.75rem;

		display: grid;
		grid-template-rows: repeat(2, auto);
		grid-auto-flow: column;
		row-gap: 0.5rem;
		column-gap: 2rem;

		.key {
			color: var(--text-light);
			font-size: 0.75rem;
			font-weight: 500;
		}

		.value {
			display: flex;
			gap: 0.5rem;
			align-items: center;

			color: var(--subscriber);

			span {
				color: var(--text);
				font-size: 0.875rem;
				font-weight: 600;
			}
		}
	}

	@media screen and (max-width: 960px) {
		.sub-grid {
			grid-template-rows: repeat(4, auto);
		}
	}
</style>
