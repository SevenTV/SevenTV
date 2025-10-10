<script lang="ts">
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";
	import { CaretLeft, User as UserIcon } from "phosphor-svelte";
	import { type User } from "$/gql/graphql";
	import UserName from "../user-name.svelte";
	import UserSearch from "../user-search.svelte";
	import { purchasePickems } from "$/lib/pickems";

	interface Props {
		mode: DialogMode;
	}

	let { mode = $bindable("hidden") }: Props = $props();

	let recipient = $state<User>();

	let giftLoading = $state(false);

	async function gift() {
		if (!recipient) {
			return;
		}

		giftLoading = true;

		await purchasePickems(undefined, recipient.id);

		giftLoading = false;
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.pickems.gift.title")}</h1>
		<hr />
		{#if recipient}
			<p>
				{$t("dialogs.pickems.gift.gift_to")} <UserName user={recipient} />.
			</p>
		{:else}
			<UserSearch
				placeholder="Search User"
				onresultclick={(e, user) => {
					e.preventDefault();
					recipient = user;
				}}
			>
				{#snippet icon()}
					<UserIcon />
				{/snippet}
				<h2>{$t("dialogs.pickems.gift.recipient")}</h2>
			</UserSearch>
		{/if}

		<div class="buttons">
			{#if recipient}
				<Button secondary onclick={() => (recipient = undefined)} style="margin-right: auto">
					{#snippet icon()}
						<CaretLeft />
					{/snippet}
					{$t("dialogs.buttons.back")}
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

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
