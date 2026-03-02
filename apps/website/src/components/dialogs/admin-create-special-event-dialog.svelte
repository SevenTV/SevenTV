<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { PencilSimple, TextAlignLeft } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TagsInput from "../input/tags-input.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	let { mode = $bindable() }: { mode: DialogMode } = $props();

	let name = $state("");
	let description = $state("");
	let tags = $state<string[]>([]);

	let loading = $state(false);

	async function submit() {
		if (loading || !name) return;

		loading = true;

		await gqlClient().mutation(
			graphql(`
				mutation AdminCreateSpecialEvent($data: CreateSpecialEventInput!) {
					specialEvents {
						create(data: $data) {
							id
						}
					}
				}
			`),
			{
				data: {
					name,
					description,
					tags,
				},
			},
		);

		loading = false;
		mode = "hidden";
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("pages.admin.special_events.create")}</h1>
		<hr />
		<TextInput placeholder="Name" bind:value={name} required>
			{#snippet icon()}
				<PencilSimple />
			{/snippet}
			{$t("pages.admin.special_events.attributes.name")}
		</TextInput>
		<TextInput placeholder="Description" bind:value={description}>
			{#snippet icon()}
				<TextAlignLeft />
			{/snippet}
			{$t("pages.admin.special_events.attributes.description")}
		</TextInput>
		<div class="tags">
			<TagsInput bind:tags>{$t("pages.admin.special_events.attributes.tags")}</TagsInput>
		</div>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		<div class="buttons">
			<Button
				primary
				icon={loading ? loadingSpinner : undefined}
				onclick={submit}
				disabled={loading || !name}
			>
				{$t("pages.admin.special_events.attributes.create")}
			</Button>
			<Button secondary onclick={() => (mode = "hidden")}
				>{$t("pages.admin.special_events.attributes.cancel")}</Button
			>
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

	.tags {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.buttons {
		display: flex;
		gap: 1rem;
	}
</style>
