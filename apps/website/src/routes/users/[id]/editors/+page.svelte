<script lang="ts">
	import Editors from "$/components/settings/editors.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { UserEditorState } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();

	let canManageEditors = $derived(
		$user &&
			data.streamed.userRequest.value.then(
				(data) =>
					$user?.id === data.id ||
					$user.permissions.user.manageAny ||
					data.editors.some(
						(editor) =>
							editor.editor?.id === $user?.id &&
							editor.state === UserEditorState.Accepted &&
							editor.permissions.user.manageEditors,
					),
			),
	);
</script>

{#await canManageEditors}
	<Spinner />
{:then canManageEditors}
	{#if canManageEditors}
		<div class="width-wrapper">
			{#await data.streamed.userRequest.value then userData}
				<Editors userId={data.id} editors={userData.editors} tab="editors" />
			{/await}
		</div>
	{:else}
		<p>You are not allowed to manage editors for this user</p>
	{/if}
{/await}

<style lang="scss">
	.width-wrapper {
		margin-inline: auto;
		width: 100%;
		max-width: 80rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
