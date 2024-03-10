<script lang="ts">
	import Button from "../input/button.svelte";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import TextInput from "../input/text-input.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog width={40} bind:mode>
	<div class="layout">
		<div class="preview">
			<EmoteSetPreview />
		</div>
		<div class="content">
			<h1>Delete Cat Emotes</h1>
			<span class="details"
				>This will delete your emote set permanently. You cannot undo this action.</span
			>
			<TextInput placeholder="Reason">
				<span class="label">Reason for deletion</span>
			</TextInput>
			<div class="buttons">
				<Button style="color: var(--danger)">Delete</Button>
				<Button secondary on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
			</div>
		</div>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1.5rem 1rem;

		display: flex;
		gap: 2rem;
	}

	.preview {
		align-self: center;

		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.content {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
