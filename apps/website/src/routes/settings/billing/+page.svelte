<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import BillingTable from "$/components/settings/billing-table.svelte";
	import PaymentMethod from "$/components/settings/payment-method.svelte";
	import SubInfo from "$/components/sub-info.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import Toggle from "$/components/input/toggle.svelte";
	import moment from "moment/min/moment-with-locales";
	import { MagnifyingGlass, Plus } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { type SubscriptionInfo } from "$/gql/graphql";

	let historyTab: "all" | "subscriptions" | "other" = $state("all");
</script>

<svelte:head>
	<title>{$t("page_titles.billing_settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<section>
	<div>
		<h2>{$t("common.your_subscription")}</h2>
		<span class="details">{$t("pages.settings.billing.subscription.details")}</span>
	</div>
	<div class="content">
		<SubInfo data={{} as SubscriptionInfo} style="padding-block: 0" />
		<hr />
		<Toggle>
			<div>
				<h3>{$t("pages.settings.billing.subscription.use_credits")}</h3>
				<span class="details">{$t("pages.settings.billing.subscription.use_credits_details")}</span>
			</div>
		</Toggle>
		<hr />
		<div>
			<h3>{$t("pages.settings.billing.subscription.cancelled")}</h3>
			<span class="details">
				{$t("pages.settings.billing.subscription.cancelled_details", {
					values: { date: moment().format("ll") },
				})}
			</span>
		</div>
	</div>
</section>

<section>
	<div>
		<h2>{$t("pages.settings.billing.history.title")}</h2>
		<span class="details">{$t("pages.settings.billing.history.details")}</span>
	</div>
	<div class="content">
		<nav class="nav-bar">
			<div class="buttons">
				<TabLink
					title={$t("labels.all")}
					matcher={() => historyTab === "all"}
					onclick={() => (historyTab = "all")}
				/>
				<TabLink
					title={$t("common.subscriptions", { values: { count: 2 } })}
					matcher={() => historyTab === "subscriptions"}
					onclick={() => (historyTab = "subscriptions")}
				/>
				<TabLink
					title={$t("labels.other")}
					matcher={() => historyTab === "other"}
					onclick={() => (historyTab = "other")}
				/>
			</div>
			<TextInput placeholder={$t("labels.search")}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput>
		</nav>
		<BillingTable />
	</div>
</section>

<section>
	<div>
		<h2>{$t("common.payment_methods", { values: { count: 2 } })}</h2>
		<span class="details">{$t("pages.settings.billing.payment_methods.details")}</span>
	</div>
	<div class="content">
		<div class="payment-methods">
			<PaymentMethod type="paypal" />
			<PaymentMethod type="mastercard" />
			<PaymentMethod type="visa" />
		</div>
		<Button primary style="align-self: flex-start">
			{#snippet icon()}
				<Plus />
			{/snippet}
			{$t("pages.settings.billing.payment_methods.add_new")}
		</Button>
	</div>
</section>

<style lang="scss">
	@use "../../../styles/settings.scss";

	h3 {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.nav-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;

		.buttons {
			display: flex;
			align-items: center;
			background-color: var(--bg-light);
			border-radius: 0.5rem;
		}
	}

	.payment-methods {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
</style>
