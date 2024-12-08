<script lang="ts">
	import TagsInput from "../input/tags-input.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import DeleteEmoteSetDialog from "./delete-emote-set-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { EmoteSet } from "$/gql/graphql";
	import { compareTags } from "$/lib/utils";
	import Spinner from "../spinner.svelte";
	import { updateName, updateTags } from "$/lib/setMutations";

	interface Props {
		mode: DialogMode;
		data: EmoteSet;
	}

	let { mode = $bindable("hidden"), data = $bindable() }: Props = $props();

	let deleteDialogMode: DialogMode = $state("hidden");

	let name = $state(data.name);
	let tags = $state(data.tags);

	let nameChanged = $derived(name !== data.name);
	let tagsChanged = $derived(!compareTags(tags, data.tags));

	let changed = $derived(nameChanged || tagsChanged);

	function onDeleteClick() {
		mode = "hidden";
		deleteDialogMode = "shown";
	}

	let loading = $state(false);

	async function submit() {
		loading = true;

		if (nameChanged) {
			const newData = await updateName(data.id, name);

			if (newData) {
				data = newData;
			}
		}

		if (tagsChanged) {
			const newData = await updateTags(data.id, tags);

			if (newData) {
				data = newData;
			}
		}

		loading = false;
		mode = "hidden";
	}
</script>

<DeleteEmoteSetDialog bind:mode={deleteDialogMode} bind:data />
<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.edit_emote_set.title")}</h1>
		<hr />
		<TextInput placeholder={$t("labels.emote_set_name")} bind:value={name}>
			<span class="label">{$t("labels.emote_set_name")}</span>
		</TextInput>
		<div class="tags">
			<TagsInput bind:tags>
				<span class="label">{$t("labels.tags")}</span>
			</TagsInput>
		</div>
		<div class="buttons">
			<Button style="color: var(--danger); margin-right: auto;" onclick={onDeleteClick}>
				{$t("labels.delete")}
			</Button>
			<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			{#snippet loadingSpinner()}
				<Spinner />
			{/snippet}
			<Button
				primary
				submit
				onclick={submit}
				disabled={!changed || loading}
				icon={loading ? loadingSpinner : undefined}
			>
				{$t("labels.save")}
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

	.tags {
		display: flex;
		flex-direction: column;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
