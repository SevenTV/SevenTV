<script lang="ts">
	import type { UserEditorPermissionsInput } from "$/gql/graphql";
	import Button from "../input/button.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		initPermissions?: UserEditorPermissionsInput;
		submit: (perms: UserEditorPermissionsInput) => Promise<void>;
	}

	const DEFAULT_PERMS = {
		superAdmin: false,
		emote: { admin: false, manage: true, create: false, transfer: false },
		emoteSet: { admin: false, manage: true, create: false },
		user: {
			admin: false,
			manageProfile: true,
			manageEditors: false,
			manageBilling: false,
			managePersonalEmoteSet: false,
		},
	};

	let { mode = $bindable(), initPermissions = DEFAULT_PERMS, submit }: Props = $props();

	let permissions = $state(initPermissions);

	let loading = $state(false);

	async function clickSubmit() {
		loading = true;
		await submit(permissions);
		loading = false;
		mode = "hidden";
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.editor.permissions")}</h1>
		<hr />
		<Checkbox bind:value={permissions.superAdmin}>{$t("dialogs.editor.super_admin")}</Checkbox>
		<div>
			<span class="label">{$t("dialogs.editor.emote_sets")}</span>
			<div class="settings">
				{#if permissions.superAdmin}
					<Checkbox value={true} disabled>{$t("dialogs.editor.admin")}</Checkbox>
				{:else}
					<Checkbox bind:value={permissions.emoteSet.admin}>{$t("dialogs.editor.admin")}</Checkbox>
				{/if}
				{#if permissions.superAdmin || permissions.emoteSet.admin}
					<Checkbox value={true} disabled>Manage</Checkbox>
					<Checkbox value={true} disabled>Create</Checkbox>
				{:else}
					<Checkbox bind:value={permissions.emoteSet.manage}>Manage</Checkbox>
					<Checkbox bind:value={permissions.emoteSet.create}>Create</Checkbox>
				{/if}
			</div>
		</div>
		<div>
			<span class="label">{$t("dialogs.editor.emotes")}</span>
			<div class="settings">
				{#if permissions.superAdmin}
					<Checkbox value={true} disabled>{$t("dialogs.editor.admin")}</Checkbox>
				{:else}
					<Checkbox bind:value={permissions.emote.admin}>{$t("dialogs.editor.admin")}</Checkbox>
				{/if}
				{#if permissions.superAdmin || permissions.emote.admin}
					<Checkbox value={true} disabled>Manage</Checkbox>
					<Checkbox value={true} disabled>Create</Checkbox>
					<Checkbox value={true} disabled>Transfer</Checkbox>
				{:else}
					<Checkbox bind:value={permissions.emote.manage}>Manage</Checkbox>
					<Checkbox bind:value={permissions.emote.create}>Create</Checkbox>
					<Checkbox bind:value={permissions.emote.transfer}>Transfer</Checkbox>
				{/if}
			</div>
		</div>
		<div>
			<span class="label">{$t("dialogs.editor.user")}</span>
			<div class="settings">
				{#if permissions.superAdmin}
					<Checkbox value={true} disabled>{$t("dialogs.editor.admin")}</Checkbox>
				{:else}
					<Checkbox bind:value={permissions.user.admin}>{$t("dialogs.editor.admin")}</Checkbox>
				{/if}
				{#if permissions.superAdmin || permissions.user.admin}
					<Checkbox value={true} disabled>Manage Billing</Checkbox>
					<Checkbox value={true} disabled>Manage Profile</Checkbox>
					<Checkbox value={true} disabled>Manage Editors</Checkbox>
					<Checkbox value={true} disabled>{$t("dialogs.editor.manage_personal_emotes")}</Checkbox>
				{:else}
					<Checkbox bind:value={permissions.user.manageBilling}>Manage Billing</Checkbox>
					<Checkbox bind:value={permissions.user.manageProfile}>Manage Profile</Checkbox>
					<Checkbox bind:value={permissions.user.manageEditors}>Manage Editors</Checkbox>
					<Checkbox bind:value={permissions.user.managePersonalEmoteSet}>
						{$t("dialogs.editor.manage_personal_emotes")}
					</Checkbox>
				{/if}
			</div>
		</div>
		<div class="buttons">
			<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			{#snippet loadingSpinner()}
				<Spinner />
			{/snippet}
			<Button
				primary
				submit
				icon={loading ? loadingSpinner : undefined}
				disabled={loading}
				onclick={clickSubmit}
			>
				{$t("dialogs.editor.confirm")}
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
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.settings {
		margin-top: 0.4rem;

		display: grid;
		grid-template-columns: auto auto;
		gap: 0.5rem;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 0.5rem;
	}
</style>
