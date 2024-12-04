<script lang="ts">
	import moment from "moment/min/moment-with-locales";
	import { Clock, CreditCard, Gift, Hourglass, IconContext, Sparkle } from "phosphor-svelte";
	import Date from "./date.svelte";
	import { t } from "svelte-i18n";
	import { SubscriptionProductKind, SubscriptionState, type SubscriptionInfo } from "$/gql/graphql";
	import type { HTMLAttributes } from "svelte/elements";
	import UserName from "./user-name.svelte";

	type Props = { data: SubscriptionInfo } & HTMLAttributes<HTMLDivElement>;

	let { data, ...restProps }: Props = $props();

	function getPlanName(info: SubscriptionInfo) {
		if (!info.activePeriod) {
			return $t("sub_info.free");
		}

		switch (info.activePeriod.subscriptionProductVariant.kind) {
			case SubscriptionProductKind.Monthly:
				return "Monthly";
			case SubscriptionProductKind.Yearly:
				return "Yearly";
		}
	}
</script>

<div class="sub-grid" {...restProps}>
	<IconContext values={{ weight: "bold", size: 1.2 * 16, color: "var(--store)" }}>
		<span class="key">{$t("sub_info.plan")}</span>
		<span class="value">
			<Sparkle />
			<span>{getPlanName(data)}</span>
		</span>

		{#if data.activePeriod?.giftedBy}
			<span class="key">{$t("sub_info.gifted_by")}</span>
			<span class="value">
				<Gift />
				<a href="/users/{data.activePeriod.giftedBy.id}">
					<UserName user={data.activePeriod.giftedBy} />
				</a>
			</span>
		{/if}

		{#if data.activePeriod}
			<span class="key"
				>{data.activePeriod.subscription.state === SubscriptionState.Active &&
				!data.activePeriod.giftedBy
					? "Next Billing"
					: $t("sub_info.ends")}</span
			>
			<span class="value">
				{#if data.activePeriod.subscription.state === SubscriptionState.Active && !data.activePeriod.giftedBy}
					<CreditCard />
				{:else}
					<Hourglass />
				{/if}
				<Date date={moment(data.activePeriod.end)} />
			</span>
		{/if}

		<span class="key">{data.activePeriod ? $t("sub_info.total") : "Previously Subscribed"}</span>
		<span class="value">
			<Clock />
			{#if data.totalDays > 0}
				<span title={moment.duration(data.totalDays, "days").humanize()}>
					{moment.duration(data.totalDays, "days").humanize({ d: Infinity })}
				</span>
			{:else}
				<span>&#60; {moment.duration(1, "days").humanize()}</span>
			{/if}
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

			a {
				text-decoration: none;
			}
		}
	}

	@media screen and (max-width: 960px) {
		.sub-grid {
			grid-template-rows: repeat(4, auto);
		}
	}
</style>
