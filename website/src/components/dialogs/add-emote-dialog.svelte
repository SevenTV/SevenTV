<script lang="ts">
	import { MagnifyingGlass } from "phosphor-svelte";
	import TextInput from "../input/text-input.svelte";
	import { DialogMode } from "./dialog.svelte";
	import Expandable from "../expandable.svelte";
	import Checkbox from "../checkbox.svelte";
	import Button from "../button.svelte";
	import Flags from "../flags.svelte";
	import EmoteDialog from "./emote-dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<EmoteDialog title="Add AlienPls to" bind:mode>
	<TextInput placeholder="Emote name" style="max-width: 12.5rem" slot="preview">
		<span class="label">Change emote name</span>
	</TextInput>
	<TextInput placeholder="Search Emote Set">
		<MagnifyingGlass slot="icon" />
	</TextInput>
	<Expandable title="Pinned">
		<Checkbox option>
			<div slot="left-label" class="emote-set">
				Emote Set 1
				<Flags flags={["600/1000", "default"]} />
			</div>
		</Checkbox>
		<Checkbox option>
			<div slot="left-label" class="emote-set">
				Emote Set 1
				<Flags flags={["600/1000"]} />
			</div>
		</Checkbox>
	</Expandable>
	<Expandable title="ayyybubu">
		<Checkbox option>
			<div slot="left-label" class="emote-set">
				Emote Set 1
				<Flags flags={["600/1000", "personal"]} />
			</div>
		</Checkbox>
		<Checkbox option>
			<div slot="left-label" class="emote-set">
				Emote Set 1
				<Flags flags={["600/1000"]} />
			</div>
		</Checkbox>
		<Checkbox option>
			<div slot="left-label" class="emote-set">
				Emote Set 1
				<Flags flags={["600/1000"]} />
			</div>
		</Checkbox>
	</Expandable>
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

	.emote-set {
		font-size: 0.875rem;
		font-weight: 500;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
