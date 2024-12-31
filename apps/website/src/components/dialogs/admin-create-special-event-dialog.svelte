<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { PencilSimple, TextAlignLeft } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TagsInput from "../input/tags-input.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";

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
		<h1>Create Special Event</h1>
		<hr />
		<TextInput placeholder="Name" bind:value={name} required>
			{#snippet icon()}
				<PencilSimple />
			{/snippet}
			Name
		</TextInput>
		<TextInput placeholder="Description" bind:value={description}>
			{#snippet icon()}
				<TextAlignLeft />
			{/snippet}
			Description
		</TextInput>
		<div class="tags">
			<TagsInput bind:tags>Tags</TagsInput>
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
				Create
			</Button>
			<Button secondary onclick={() => (mode = "hidden")}>Cancel</Button>
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
