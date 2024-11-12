<script lang="ts">
	import { t } from "svelte-i18n";
	import Connections from "$/components/profile/connections.svelte";
	import type { PageData } from "./$types";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import { UserEditorState } from "$/gql/graphql";
	import Spinner from "$/components/spinner.svelte";

	let { data }: { data: PageData } = $props();
</script>

<div class="layout">
	{#await data.streamed.userRequest}
		<Spinner />
	{:then user}
		{#if user.connections.length > 0}
			<h2>{$t("common.connections")}</h2>
			<div class="link-list">
				<Connections {user} big />
			</div>
		{/if}
		{#if user.editors.some((e) => e.editor && e.state === UserEditorState.Accepted)}
			<h2>{$t("common.editors")}</h2>
			<div class="link-list">
				{#each user.editors as editor}
					{#if editor.editor && editor.state === UserEditorState.Accepted}
						<ChannelPreview big size={1.5} user={editor.editor} />
					{/if}
				{/each}
			</div>
		{/if}
	{/await}
</div>

<style lang="scss">
	.layout {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h2 {
		font-size: 1rem;
		font-weight: 600;
	}

	.link-list {
		display: flex;
		flex-direction: column;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}
</style>
