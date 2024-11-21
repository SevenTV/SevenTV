<script lang="ts">
	import { EnvelopeSimple, EnvelopeSimpleOpen, Gear } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Notification from "./notification.svelte";
	import { t } from "svelte-i18n";

	let { popup = false }: { popup?: boolean } = $props();

	let read = $state(false);
</script>

<div class="container" class:popup>
	<div class="header">
		<h1>{$t("common.notifications")}</h1>
		<Button onclick={() => (read = !read)}>
			{#snippet icon()}
				{#if read}
					<EnvelopeSimple />
				{:else}
					<EnvelopeSimpleOpen />
				{/if}
			{/snippet}
		</Button>
		<Button>
			{#snippet icon()}
				<Gear />
			{/snippet}
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
		scrollbar-gutter: stable;
	}
</style>
