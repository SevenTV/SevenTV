<script lang="ts">
	import moment from "moment/min/moment-with-locales";
	import { Clock, Coin, Gift, Hourglass, IconContext, Sparkle } from "phosphor-svelte";
	import Date from "./date.svelte";
	import { t } from "svelte-i18n";
</script>

<div class="sub-grid" {...$$restProps}>
	<IconContext values={{ weight: "bold", size: 1.2 * 16, color: "var(--store)" }}>
		<span class="key">{$t("sub_info.plan")}</span>
		<span class="value">
			<Sparkle />
			<span>{$t("sub_info.free")}</span>
		</span>

		<span class="key">{$t("sub_info.gifted_by")}</span>
		<span class="value">
			<Gift />
			<span>Forsen</span>
		</span>

		<span class="key">{$t("sub_info.ends")}</span>
		<span class="value">
			<Hourglass />
			<Date date={moment()} />
		</span>

		<span class="key">{$t("sub_info.total")}</span>
		<span class="value">
			<Clock />
			<span>{moment.duration(317, "days").humanize({ d: Infinity })}</span>
		</span>

		<!-- <span class="key">{$t("sub_info.credits")}</span>
		<span class="value">
			<Coin />
			<span>3</span>
		</span> -->
	</IconContext>
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

			font-size: 0.875rem;
			font-weight: 600;
		}
	}

	@media screen and (max-width: 960px) {
		.sub-grid {
			grid-template-rows: repeat(4, auto);
		}
	}
</style>
