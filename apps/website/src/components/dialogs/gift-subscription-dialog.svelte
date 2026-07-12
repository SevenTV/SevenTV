<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";
	import { CaretLeft, Minus, Plus, User as UserIcon, X } from "phosphor-svelte";
	import {
		SubscriptionProductKind,
		type SubscriptionProductVariant,
		type User,
	} from "$/gql/graphql";
	import UserName from "../user-name.svelte";
	import { priceFormat } from "$/lib/utils";
	import UserSearch from "../user-search.svelte";

	interface Props {
		mode: DialogMode;
		variant: SubscriptionProductVariant;
	}

	let { mode = $bindable("hidden"), variant }: Props = $props();

	// Keep these in sync with `MAX_GIFT_RECIPIENTS`/`MAX_MONTHS` in
	// apps/api/src/http/egvault/metadata.rs and apps/api/src/stripe_common.rs
	const MAX_RECIPIENTS = 10;
	const MAX_MONTHS = 24;

	type Recipient = { user: User; months: number };

	// Each recipient can be gifted a different number of months (e.g. 3
	// months for one friend, 1 month for another) in the same checkout.
	let recipients = $state<Recipient[]>([]);
	let showSearch = $state(true);

	let giftLoading = $state(false);

	// Only the monthly plan supports a custom month count; the yearly plan is
	// always a fixed 12 month bundle for every recipient.
	let monthsEditable = $derived(variant.kind === SubscriptionProductKind.Monthly);

	function addRecipient(user: User) {
		if (recipients.length >= MAX_RECIPIENTS || recipients.some((r) => r.user.id === user.id)) {
			return;
		}

		recipients = [...recipients, { user, months: 1 }];
		showSearch = false;
	}

	function removeRecipient(user: User) {
		recipients = recipients.filter((r) => r.user.id !== user.id);
	}

	function incrementMonths(user: User) {
		recipients = recipients.map((r) =>
			r.user.id === user.id ? { ...r, months: Math.min(r.months + 1, MAX_MONTHS) } : r,
		);
	}

	function decrementMonths(user: User) {
		recipients = recipients.map((r) =>
			r.user.id === user.id ? { ...r, months: Math.max(r.months - 1, 1) } : r,
		);
	}

	let totalAmount = $derived(
		recipients.reduce((sum, r) => sum + (variant.price.amount / 100) * r.months, 0),
	);
	let totalPrice = $derived(priceFormat(variant.price.currency).format(totalAmount));

	// For the yearly plan, each recipient's `months` is always `1` (meaning
	// "1 year unit" - the backend maps that to a 12 month period), not a
	// literal month count. So the total needs a different unit/label
	// depending on the plan: sum of months for the monthly plan, sum of
	// year-units for the yearly plan.
	let totalUnits = $derived(recipients.reduce((sum, r) => sum + r.months, 0));
	let totalUnitLabel = $derived(
		monthsEditable ? $t("dialogs.gift.months_label") : $t("dialogs.gift.years_label"),
	);

	async function gift() {
		if (recipients.length === 0) {
			return;
		}

		giftLoading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation GiftSubscriptions(
						$recipients: [GiftRecipientInput!]!
						$variantId: StripeProductId!
					) {
						giftSubscriptions(recipients: $recipients, variantId: $variantId) {
							checkoutUrl
						}
					}
				`),
				{
					recipients: recipients.map((r) => ({ userId: r.user.id, months: r.months })),
					variantId: variant.id,
				},
			)
			.toPromise();

		if (res.data) {
			window.location.href = res.data.giftSubscriptions.checkoutUrl;
		}

		giftLoading = false;
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.gift.title")}</h1>
		<hr />

		{#if recipients.length > 0}
			<div class="recipients">
				{#each recipients as recipient (recipient.user.id)}
					<div class="recipient-chip">
						<UserName user={recipient.user} />

						<div class="recipient-controls">
							{#if monthsEditable}
								<div class="stepper">
									<Button
										secondary
										onclick={() => decrementMonths(recipient.user)}
										disabled={recipient.months <= 1}
									>
										{#snippet icon()}
											<Minus />
										{/snippet}
									</Button>
									<span class="months-value">{recipient.months}</span>
									<Button
										secondary
										onclick={() => incrementMonths(recipient.user)}
										disabled={recipient.months >= MAX_MONTHS}
									>
										{#snippet icon()}
											<Plus />
										{/snippet}
									</Button>
								</div>
							{/if}

							<Button secondary onclick={() => removeRecipient(recipient.user)}>
								{#snippet icon()}
									<X />
								{/snippet}
							</Button>
						</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if recipients.length < MAX_RECIPIENTS}
			{#if !showSearch}
				<Button secondary onclick={() => (showSearch = true)}>
					{#snippet icon()}
						<UserIcon />
					{/snippet}
					{$t("dialogs.gift.add_recipient")}
				</Button>
			{:else}
				<UserSearch
					placeholder={$t("labels.search_user")}
					onresultclick={(e, user) => {
						e.preventDefault();
						addRecipient(user);
					}}
				>
					{#snippet icon()}
						<UserIcon />
					{/snippet}
					<h2>{$t("dialogs.gift.recipient")}</h2>
				</UserSearch>
			{/if}
		{:else}
			<p class="hint">
				{$t("dialogs.gift.max_recipients_reached", { values: { max: MAX_RECIPIENTS } })}
			</p>
		{/if}

		{#if recipients.length > 0}
			<p class="total">
				{$t("dialogs.gift.total")}: <strong>{totalPrice}</strong>
				<span class="hint">
					({totalUnits}
					{totalUnitLabel.toLowerCase()} · {recipients.length}
					{recipients.length > 1
						? $t("dialogs.gift.recipients_label")
						: $t("dialogs.gift.recipient_label")})
				</span>
			</p>
		{/if}

		<div class="buttons">
			<Button secondary onclick={() => (mode = "hidden")} style="margin-right: auto">
				{#snippet icon()}
					<CaretLeft />
				{/snippet}
				{$t("labels.cancel")}
			</Button>

			{#snippet spinnerIcon()}
				<Spinner />
			{/snippet}

			<Button
				icon={giftLoading ? spinnerIcon : undefined}
				disabled={giftLoading || recipients.length === 0}
				onclick={gift}
				primary
				submit
			>
				{$t("dialogs.buttons.continue")}
			</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		height: 100%;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	h2 {
		font-size: 1rem;
		font-weight: 400;
	}

	.stepper {
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}

	.months-value {
		min-width: 1.25rem;
		text-align: center;
		font-weight: 600;
		font-size: 0.875rem;
	}

	.recipients {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-height: 14rem;
		overflow-y: auto;
	}

	.recipient-chip {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;

		background-color: var(--secondary);
		border-radius: 0.5rem;
	}

	.recipient-controls {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.total {
		font-size: 0.875rem;
	}

	.hint {
		color: var(--text-light);
		font-size: 0.8rem;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
