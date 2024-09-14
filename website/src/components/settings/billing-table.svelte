<script lang="ts">
	import moment from "moment/min/moment-with-locales";
	import Button from "../input/button.svelte";
	import Date from "../date.svelte";
	import { Download, PaintBrush, Star } from "phosphor-svelte";
	import PaymentBrand from "../icons/payment-brand.svelte";
	import { priceFormat } from "$/lib/utils";
	import Flags from "../flags.svelte";
	import { t } from "svelte-i18n";
</script>

<div class="scroll">
	<table>
		<thead>
			<tr>
				<th>{$t("common.date")}</th>
				<th>{$t("common.items", { values: { count: 2 } })}</th>
				<th>{$t("common.payment_methods", { values: { count: 1 } })}</th>
				<th>{$t("pages.settings.billing.table.status")}</th>
				<th>{$t("pages.settings.billing.table.amount")}</th>
				<th>{$t("pages.settings.billing.table.invoice")}</th>
			</tr>
		</thead>
		<tbody>
			{#each Array(3) as _, i}
				<tr>
					<td class="date">
						<Date date={moment("2024-01-22").subtract(i, "months")} />
					</td>
					<td>
						<div class="items">
							<div class="item">
								<Star color="var(--store)" />
								<span>Subscription - Tier 1</span>
							</div>
							<div class="item">
								<PaintBrush color="var(--store)" />
								<span>Paint Bundle - Candy</span>
							</div>
						</div>
					</td>
					<td>
						<PaymentBrand type="paypal" />
					</td>
					<td>
						<Flags flags={["paid"]} />
					</td>
					<td class="amount">
						{priceFormat().format(5.99 + i * 2.5)}
					</td>
					<td class="shrink">
						<Button on:click={() => alert("download invoice")}>
							<Download slot="icon" />
						</Button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style lang="scss">
	.scroll {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;
	}

	td {
		padding: 1rem 1.5rem;
		vertical-align: top;
	}

	.date {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 1rem;

		.item {
			display: flex;
			align-items: center;
			gap: 0.75rem;
		}
	}
</style>
