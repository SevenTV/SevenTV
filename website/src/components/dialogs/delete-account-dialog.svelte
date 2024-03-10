<script lang="ts">
	import { Warning } from "phosphor-svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Checkbox from "../input/checkbox.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>Delete Account</h1>
		<hr />
		<div class="warning">
			<Warning size="4rem" weight="fill" color="var(--danger)" />
			<h2>Warning</h2>
			<span class="details"
				>This action will delete your account permanently. You cannot undo this action.</span
			>
		</div>
		<div class="reasons">
			<span class="label">Choose the reasons for account deletion</span>
			<Checkbox option>
				<span class="label" slot="left-label">Forsen</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">moo</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">oom</span>
			</Checkbox>
			<Checkbox option>
				<span class="label" slot="left-label">Need more money for gamba</span>
			</Checkbox>
		</div>
		<TextInput placeholder="ayyybubu">
			<span class="label">Enter your username to confirm</span>
		</TextInput>
		<div class="buttons">
			<Button style="color: var(--danger)" submit>Delete</Button>
			<Button secondary on:click={() => (mode = DialogMode.Hidden)}>Cancel</Button>
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

	h2 {
		font-size: 1.5rem;
		font-weight: 500;
	}

	.details {
		font-size: 0.875rem;
		color: var(--text-light);
	}

	.warning {
		display: flex;
		flex-direction: column;
		align-items: center;

		text-align: center;
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
