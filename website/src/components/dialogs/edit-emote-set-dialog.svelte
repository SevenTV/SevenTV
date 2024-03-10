<script lang="ts">
	import TagsInput from "../input/tags-input.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import DeleteEmoteSetDialog from "./delete-emote-set-dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;

	let deleteDialogMode = DialogMode.Hidden;

	function onDeleteClick() {
		mode = DialogMode.Hidden;
		deleteDialogMode = DialogMode.Shown;
	}
</script>

<DeleteEmoteSetDialog bind:mode={deleteDialogMode} />
<Dialog bind:mode>
	<div class="layout">
		<h1>Edit Emote Set</h1>
		<hr />
		<TextInput placeholder="Emote Set name">
			<span class="label">Emote set name</span>
		</TextInput>
		<div class="tags">
			<TagsInput>
				<span class="label">Tags</span>
			</TagsInput>
		</div>
		<div>
			<span class="label">Settings</span>
			<div class="settings">
				<Checkbox>Show on Profile</Checkbox>
				<Checkbox>Publicly Listed</Checkbox>
			</div>
		</div>
	</div>
	<div class="buttons">
		<Button style="color: var(--danger); margin-right: auto;" on:click={onDeleteClick}
			>Delete</Button
		>
		<Button secondary on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
		<Button primary>Save</Button>
	</div>
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

	.settings {
		margin-top: 0.4rem;

		display: grid;
		grid-template-columns: auto auto;
		gap: 0.5rem;
	}

	.buttons {
		margin-top: auto;
		padding: 1rem;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
