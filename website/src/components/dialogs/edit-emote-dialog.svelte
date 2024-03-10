<script lang="ts">
	import { User } from "phosphor-svelte";
	import TagsInput from "../input/tags-input.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import TextInput from "../input/text-input.svelte";
	import Checkbox from "../checkbox.svelte";
	import Button from "../button.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog bind:mode>
	<div class="layout">
		<h1>Edit AlienPls</h1>
		<hr />
		<TextInput placeholder="Emote name">
			<span class="label">Emote name</span>
		</TextInput>
		<div class="tags">
			<TagsInput>
				<span class="label">Tags</span>
			</TagsInput>
		</div>
		<TextInput placeholder="Search users">
			<span class="label">Emote Attribution</span>
			<User slot="icon" />
		</TextInput>
		<div>
			<span class="label">Settings</span>
			<div class="settings">
				<Checkbox>Overlaying</Checkbox>
				<Checkbox>Listed</Checkbox>
				<Checkbox>Private</Checkbox>
				<Checkbox>Personal Use</Checkbox>
			</div>
		</div>
	</div>
	<div class="buttons">
		<Button style="color: var(--error); margin-right: auto;">Delete</Button>
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
