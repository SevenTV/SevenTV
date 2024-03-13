<script lang="ts">
	import { EnvelopeSimple, EnvelopeSimpleOpen, Gear, MagnifyingGlass, X } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import MessagePreview from "./message-preview.svelte";

	export let popup = false;

	let read = false;
</script>

<div class="dms" class:popup>
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
	<TextInput placeholder="Search" style="margin: 0 1rem">
		<MagnifyingGlass slot="icon" />
	</TextInput>
	<div class="messages" class:scrollable={popup}>
		{#each Array(20) as _, i}
			<MessagePreview />
			{#if i !== 9}
				<hr />
			{/if}
		{/each}
	</div>
</div>

<style lang="scss">
	.dms {
		min-width: 25rem;
		max-width: 50rem;
		height: 100%;

		margin-inline: auto;

		padding-top: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		&.popup {
			max-height: 80vh;
		}
	}

	.header {
		padding: 0 1rem;

		display: flex;
		align-items: center;
		gap: 0.5rem;

		h1 {
			margin-right: auto;

			font-size: 1rem;
			font-weight: 600;
		}
	}

	.messages {
		padding: 0 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.scrollable {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;
	}
</style>
