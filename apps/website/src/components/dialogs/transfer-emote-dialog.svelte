<script lang="ts">
	import { User as UserIcon } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Emote, User } from "$/gql/graphql";
	import Spinner from "../spinner.svelte";
	import ChannelPreview from "../channel-preview.svelte";
	import { updateOwner } from "$/lib/emoteMutations";
	import UserSearch from "../user-search.svelte";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data = $bindable() }: Props = $props();

	let recipient = $state<User>();

	let loading = $state(false);

	async function submit() {
		if (!recipient) {
			return;
		}

		loading = true;

		const newData = await updateOwner(data.id, recipient.id);

		if (newData) {
			data = newData;
		}

		loading = false;
		mode = "hidden";
	}
</script>

<EmoteDialog
	width={35}
	title={$t("dialogs.transfer_emote.title", { values: { emote: data.defaultName } })}
	bind:mode
	{data}
>
	<span class="details">
		{$t("dialogs.transfer_emote.details", { values: { emote: data.defaultName } })}
	</span>
	{#if recipient}
		<span class="label">{$t("dialogs.transfer_emote.receipient")}</span>
		<ChannelPreview
			user={recipient}
			size={2}
			onclick={(e) => {
				e.preventDefault();
				recipient = undefined;
			}}
		/>
	{:else}
		<UserSearch
			placeholder={$t("labels.search_users", { values: { count: 1 } })}
			onResultClick={(e, user) => {
				e.preventDefault();
				recipient = user;
			}}
		>
			{#snippet icon()}
				<UserIcon />
			{/snippet}
			<span class="label">{$t("dialogs.transfer_emote.receipient")}</span>
		</UserSearch>
	{/if}
	{#snippet buttons()}
		<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		<Button
			primary
			submit
			disabled={loading}
			onclick={submit}
			icon={loading ? loadingSpinner : undefined}
		>
			{$t("dialogs.transfer_emote.transfer")}
		</Button>
	{/snippet}
</EmoteDialog>

<style lang="scss">
	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
