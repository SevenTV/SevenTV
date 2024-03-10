<script lang="ts">
	import { User } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import { DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<EmoteDialog width={35} title="Transfer AlienPls" bind:mode>
	<span class="details">This will transfer AlienPls.</span>
	<TextInput placeholder="Enter Username">
		<span class="label">Emote Receipient</span>
		<User slot="icon" />
	</TextInput>
	<svelte:fragment slot="buttons">
		<Button on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
		<Button primary>Transfer</Button>
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
