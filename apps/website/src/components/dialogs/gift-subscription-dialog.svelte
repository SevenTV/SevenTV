<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";
	import { CaretLeft, User as UserIcon } from "phosphor-svelte";
	import { type SubscriptionProductVariant, type User } from "$/gql/graphql";
	import UserName from "../user-name.svelte";
	import { variantUnit } from "$/lib/utils";
	import UserSearch from "../user-search.svelte";

	interface Props {
		mode: DialogMode;
		variant: SubscriptionProductVariant;
	}

	let { mode = $bindable("hidden"), variant }: Props = $props();

	let recipient = $state<User>();

	let giftLoading = $state(false);

	async function gift() {
		if (!recipient) {
			return;
		}

		giftLoading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation Subscribe($userId: Id!, $variantId: ProductId!) {
						billing(userId: $userId) {
							subscribe(variantId: $variantId) {
								checkoutUrl
							}
						}
					}
				`),
				{ userId: recipient.id, variantId: variant.id },
			)
			.toPromise();

		if (res.data) {
			window.location.href = res.data.billing.subscribe.checkoutUrl;
		}

		giftLoading = false;
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>Gift 1 {variantUnit(variant)}</h1>
		<hr />
		{#if recipient}
			<p>
				Gift 1 {variantUnit(variant)} to <UserName user={recipient} />.
			</p>
		{:else}
			<UserSearch
				placeholder="Search User"
				onResultClick={(e, user) => {
					e.preventDefault();
					recipient = user;
				}}
			>
				{#snippet icon()}
					<UserIcon />
				{/snippet}
				<h2>Select recipient</h2>
			</UserSearch>
		{/if}

		<div class="buttons">
			{#if recipient}
				<Button secondary onclick={() => (recipient = undefined)} style="margin-right: auto">
					{#snippet icon()}
						<CaretLeft />
					{/snippet}
					Back
				</Button>
			{/if}

			{#if !recipient}
				<Button secondary onclick={() => (mode = "hidden")} style="margin-left: auto"
					>{$t("labels.cancel")}</Button
				>
			{/if}

			{#snippet spinnerIcon()}
				<Spinner />
			{/snippet}

			<Button
				icon={giftLoading ? spinnerIcon : undefined}
				disabled={giftLoading || !recipient}
				onclick={gift}
				primary
				submit
			>
				Continue
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

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
