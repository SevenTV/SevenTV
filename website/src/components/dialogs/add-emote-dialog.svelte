<script lang="ts">
	import { MagnifyingGlass } from "phosphor-svelte";
	import EmotePreview from "../emote-preview.svelte";
	import TextInput from "../input/text-input.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import Expandable from "../expandable.svelte";
	import Checkbox from "../checkbox.svelte";
	import Button from "../button.svelte";
	import Flags from "../flags.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog width={45} bind:mode>
	<div class="layout">
		<div class="preview">
			<EmotePreview />
			<TextInput placeholder="Emote name" style="max-width: 12.5rem">
				<span class="label">Change emote name</span>
			</TextInput>
		</div>
		<div class="emote-sets">
			<h1>Add AlienPls to</h1>
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
		</div>
		<div class="buttons">
			<Button on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
			<Button primary>Confirm</Button>
		</div>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1.5rem 1rem;

		display: grid;
		grid-template-columns: auto 1fr;
		gap: 2rem;
		align-items: center;
	}

	.preview {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.emote-sets {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.emote-set {
		font-size: 0.875rem;
		font-weight: 500;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
