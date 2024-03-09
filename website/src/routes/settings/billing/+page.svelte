<script lang="ts">
	import Button from "$/components/button.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import BillingTable from "$/components/settings/billing-table.svelte";
	import PaymentMethod from "$/components/settings/payment-method.svelte";
	import SubInfo from "$/components/sub-info.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import Toggle from "$/components/toggle.svelte";
	import moment from "moment";
	import { MagnifyingGlass, Plus } from "phosphor-svelte";

	let historyTab: "all" | "subscriptions" | "other" = "all";
</script>

<svelte:head>
	<title>Billing Settings - 7TV</title>
</svelte:head>

<section>
	<div>
		<h2>Your Subscription</h2>
		<span class="details"
			>Lorem ipsum dolor sit, amet consectetur adipisicing elit. Magni incidunt numquam itaque, amet
			ad, pariatur possimus illo atque sunt asperiores molestias autem ex nobis consequuntur culpa.
			Corporis possimus aspernatur soluta.</span
		>
	</div>
	<div class="content">
		<SubInfo style="padding-block: 0" />
		<hr />
		<Toggle>
			<div>
				<h3>Use Subscription Credits</h3>
				<span class="details"
					>Your subscription credits will be used first for your next billing</span
				>
			</div>
		</Toggle>
		<hr />
		<div>
			<h3>Subscription Cancelled</h3>
			<span class="details">Your subscription benefits will expire on {moment().format("ll")}</span>
		</div>
	</div>
</section>

<section>
	<div>
		<h2>Billing History</h2>
		<span class="details"
			>See all payment activities associated with your account, such as past billing statements and
			transactions</span
		>
	</div>
	<div class="content">
		<nav class="nav-bar">
			<div class="buttons">
				<TabLink
					title="All"
					matcher={() => historyTab === "all"}
					on:click={() => (historyTab = "all")}
				/>
				<TabLink
					title="Subscriptions"
					matcher={() => historyTab === "subscriptions"}
					on:click={() => (historyTab = "subscriptions")}
				/>
				<TabLink
					title="Other"
					matcher={() => historyTab === "other"}
					on:click={() => (historyTab = "other")}
				/>
			</div>
			<TextInput placeholder="Search">
				<MagnifyingGlass slot="icon" />
			</TextInput>
		</nav>
		<BillingTable />
	</div>
</section>

<section>
	<div>
		<h2>Payment Methods</h2>
		<span class="details">Check and update your preferred payment methods</span>
	</div>
	<div class="content">
		<div class="payment-methods">
			<PaymentMethod type="paypal" />
			<PaymentMethod type="mastercard" />
			<PaymentMethod type="visa" />
		</div>
		<Button primary style="align-self: flex-start">
			<Plus slot="icon" />
			Add New
		</Button>
	</div>
</section>

<style lang="scss">
	@import "../../../styles/settings.scss";

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
