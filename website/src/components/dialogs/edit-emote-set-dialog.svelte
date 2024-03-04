<script lang="ts">
	import TagsInput from "../input/tags-input.svelte";
	import Dialog from "./dialog.svelte";
	import Checkbox from "../checkbox.svelte";
	import Button from "../button.svelte";
	import { createEventDispatcher } from "svelte";

	const dispatch = createEventDispatcher();
</script>

<Dialog on:close>
	<div class="layout">
		<h1>Edit Emote Set</h1>
		<hr />
		<label>
			<span>Emote Set Name</span>
			<input type="text" placeholder="Emote set name" />
		</label>
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<span>Tags</span>
			<TagsInput />
		</label>
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<span>Settings</span>
			<div class="settings">
				<Checkbox label="Show on Profile" />
				<Checkbox label="Publicly Listed" />
			</div>
		</label>
	</div>
    <div class="buttons">
        <Button style="color: var(--error); margin-right: auto;">Delete</Button>
        <Button secondary on:click={() => dispatch("close")}>Cancel</Button>
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

	label {
		display: flex;
		flex-direction: column;

		& > span {
			margin-bottom: 0.4rem;
			font-size: 0.875rem;
			font-weight: 500;
		}
	}

	.settings {
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
