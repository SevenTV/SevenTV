<script lang="ts">
	import { graphql } from "$/gql";
	import type { KickLinkInput } from "$/gql/graphql";
	import { currentError, errorDialogMode } from "$/lib/error";
	import { gqlClient } from "$/lib/gql";
	import KickLogo from "../icons/kick-logo.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	let { mode = $bindable(), userId }: { mode: DialogMode; userId: string } = $props();

	let loading = $state(false);

	let username = $state<string>();

	async function getKickDetails(username: string): Promise<KickLinkInput | undefined> {
		const res = await fetch(`https://kick.com/api/v2/channels/${username}`);

		if (res.ok) {
			const data = await res.json();

			return {
				id: data.user_id.toString(),
				username: data.slug,
				displayName: data.user.username,
				avatarUrl: data.user.profile_pic,
			};
		} else {
			currentError.set(`Failed to upload profile picture: ${res.statusText}`);
			errorDialogMode.set("shown");
			return undefined;
		}
	}

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (!username) return;

		loading = true;

		const kickChannel = await getKickDetails(username);

		if (!kickChannel) {
			loading = false;
			return;
		}

		await gqlClient().mutation(
			graphql(`
				mutation AdminLinkKick($userId: Id!, $kickChannel: KickLinkInput!) {
					users {
						user(id: $userId) {
							manuallyLinkKick(kickChannel: $kickChannel) {
								id
							}
						}
					}
				}
			`),
			{ userId, kickChannel },
		);

		loading = false;
		mode = "hidden";
	}
</script>

<Dialog bind:mode>
	<form onsubmit={submit} class="layout">
		<h1>{$t("pages.admin.users.id.actions.connections.kick")}</h1>
		<hr />
		<TextInput bind:value={username} required>
			{#snippet icon()}
				<KickLogo />
			{/snippet}
			{$t("pages.admin.users.id.actions.connections.username")}
		</TextInput>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		<div class="buttons">
			<Button submit primary icon={loading ? loadingSpinner : undefined}>Create</Button>
			<Button secondary onclick={() => (mode = "hidden")}>Close</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.buttons {
		display: flex;
		gap: 1rem;
	}
</style>
