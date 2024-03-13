<script lang="ts">
	import { EnvelopeSimple, EnvelopeSimpleOpen, Gear } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Notification from "./notification.svelte";

	export let popup = false;

	let read = false;
</script>

<div class="container" class:popup>
	<div class="header">
		<h1>Notifications</h1>
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
	<div class="notifications" class:scrollable={popup}>
		{#each Array(10) as _, i}
			<Notification />
			{#if i !== 9}
				<hr />
			{/if}
		{/each}
	</div>
</div>

<style lang="scss">
	.container {
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

	.notifications {
		padding: 0 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.scrollable {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;
	}
</style>
