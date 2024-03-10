<script lang="ts">
	import Dialog, { DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Checkbox from "../input/checkbox.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog width={40} bind:mode>
	<div class="layout">
		<h1>Report Emote</h1>
		<hr />
		<div class="reasons">
			<span class="label">Choose the reason</span>
			<Checkbox option>
				<span class="label" slot="left-label"
					>I made this emote but it was uploaded by someone else</span
				>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">This emote is a duplicate</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label"
					>This emote contains pornographic or overly sexualized imagery</span
				>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">This emote displays extreme violence or gore</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">This emote depicts me and I don't like it</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">I find this emote offensive</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">Other</span>
			</Checkbox>
		</div>
		<TextInput type="textarea" placeholder="Enter text">
			<span class="label">Additional Info</span>
		</TextInput>
		<span class="details">Abuse of the report feature may lead to your access being revoked.</span>
		<div class="buttons">
			<Button on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
			<Button primary>Send Report</Button>
		</div>
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

	.details {
		font-size: 0.75rem;
		color: var(--text-light);
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.reasons {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.buttons {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 0.5rem;
	}
</style>
