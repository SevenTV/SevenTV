<script lang="ts">
	import { UserEditorState, type User } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import { Lightning } from "phosphor-svelte";
	import ActiveEmoteSetDialog from "../dialogs/active-emote-set-dialog.svelte";
	import type { DialogMode } from "../dialogs/dialog.svelte";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";

	let { userData = $bindable() }: { userData: Promise<User> } = $props();

	let activeEmoteSetDialog: DialogMode = $state("hidden");

	function activeEmoteSetClick(e: MouseEvent) {
		e.preventDefault();
		activeEmoteSetDialog = "shown";
	}
</script>

<ActiveEmoteSetDialog bind:userData bind:mode={activeEmoteSetDialog} />

{#await userData}
	<Button secondary disabled>
		{#snippet icon()}
			<Lightning />
		{/snippet}
		<Spinner />
	</Button>
{:then userData}
	{@const hasPermission =
		$user?.permissions.user.manageAny ||
		$user?.id === userData.id ||
		userData.editors.some(
			(editor) =>
				editor.editorId === $user?.id &&
				editor.state === UserEditorState.Accepted &&
				editor.permissions.user.manageProfile,
		)}
	<Button
		secondary
		onclick={hasPermission ? activeEmoteSetClick : undefined}
		href={userData.style.activeEmoteSet
			? `/emote-sets/${userData.style.activeEmoteSet.id}`
			: undefined}
	>
		{#snippet icon()}
			<Lightning />
		{/snippet}
		{userData?.style.activeEmoteSet?.name ?? "Active Set"}
	</Button>
{/await}
