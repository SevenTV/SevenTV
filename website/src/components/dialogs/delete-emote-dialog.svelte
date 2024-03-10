<script lang="ts">
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import { DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<EmoteDialog width={35} title="Delete AlienPls" bind:mode>
	<span class="details">This will delete your emote permanently. You cannot undo this action.</span>
	<TextInput placeholder="Reason">
		<span class="label">Reason for deletion</span>
	</TextInput>
	<svelte:fragment slot="buttons">
		<Button style="color: var(--error)">Delete</Button>
		<Button secondary on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
	</svelte:fragment>
</EmoteDialog>

<style lang="scss">
	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
