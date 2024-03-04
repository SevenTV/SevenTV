<script lang="ts">
	import { User } from "phosphor-svelte";
	import TagsInput from "../input/tags-input.svelte";
	import Dialog from "./dialog.svelte";
	import SearchBar from "../input/search-bar.svelte";
	import Checkbox from "../checkbox.svelte";
	import Button from "../button.svelte";
	import { createEventDispatcher } from "svelte";

	const dispatch = createEventDispatcher();
</script>

<Dialog on:close>
	<div class="layout">
		<h1>Edit Emote</h1>
		<hr />
		<label>
			<span>Emote Name</span>
			<input type="text" placeholder="Emote name" />
		</label>
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<span>Tags</span>
			<TagsInput />
		</label>
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<span>Emote Attribution</span>
			<SearchBar placeholder="Enter username" grow>
				<User />
			</SearchBar>
		</label>
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<span>Settings</span>
			<div class="settings">
				<Checkbox label="Overlaying" />
				<Checkbox label="Listed" />
				<Checkbox label="Private" />
				<Checkbox label="Personal Use" />
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
