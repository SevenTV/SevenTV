<script lang="ts">
	import { EnvelopeSimple, EnvelopeSimpleOpen, Gear, MagnifyingGlass, X } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import MessagePreview from "./message-preview.svelte";

	let read = false;
</script>

<div class="popup">
	<div class="header">
		<h1>Direct Messages</h1>
		<Button title="Mark all as read" on:click={() => (read = !read)}>
			<svelte:fragment slot="icon">
				{#if read}
					<EnvelopeSimple />
				{:else}
					<EnvelopeSimpleOpen />
				{/if}
			</svelte:fragment>
		</Button>
		<Button>
			<Gear slot="icon" />
		</Button>
	</div>
	<TextInput placeholder="Search">
		<MagnifyingGlass slot="icon" />
	</TextInput>
	<div class="previews">
		{#each Array(10) as _, i}
			<MessagePreview />
			{#if i !== 9}
				<hr />
			{/if}
		{/each}
	</div>
</div>

<style lang="scss">
	.popup {
		min-width: 25rem;
		max-width: 50rem;
		margin-inline: auto;
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.header {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		h1 {
			margin-right: auto;
		}
	}

	.previews {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}
</style>
