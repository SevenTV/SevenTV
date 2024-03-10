<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import EmoteSetPicker from "../emote-set-picker.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<EmoteDialog title="Add AlienPls to" bind:mode>
	<TextInput placeholder="Emote name" style="max-width: 12.5rem" slot="preview">
		<span class="label">Change emote name</span>
	</TextInput>
	<EmoteSetPicker />
	<svelte:fragment slot="buttons">
		<Button on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
		<Button primary>Confirm</Button>
	</svelte:fragment>
</EmoteDialog>

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
