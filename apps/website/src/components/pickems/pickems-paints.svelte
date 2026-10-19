<script lang="ts">
	import type { Paint } from "$/gql/graphql";
	import { onMount } from "svelte";
	import PaintComponent from "../paint.svelte";
	import { fly } from "svelte/transition";
	import { user } from "$/lib/auth";
	import { t } from "svelte-i18n";

	let { paints }: { paints: Paint[] } = $props();
	let username = $derived($user?.mainConnection?.platformDisplayName ?? "Username");

	let paintMouseOver = $state(false);
	let paintIndex = $state(0);
	onMount(() => {
		const interval = setInterval(() => {
			paintIndex = (paintIndex + 1) % paints.length;
		}, 5000);
		return () => clearInterval(interval);
	});
</script>

<div class="paint">
	{#each paints as paint, i}
		{#if paintIndex === i}
			<div
				class="paint-inner"
				in:fly={{ y: 100, duration: 500 }}
				out:fly={{ y: -100, duration: 500 }}
			>
				<PaintComponent
					{paint}
					enableDialog
					onmouseenter={() => (paintMouseOver = true)}
					onmouseleave={() => (paintMouseOver = false)}
				>
					<h2>
						{#if paintMouseOver && username}
							{username}
						{:else}
							{paint.name}
						{/if}
					</h2>
				</PaintComponent>
			</div>
		{/if}
	{/each}
</div>

<style lang="scss">
	.paint {
		position: relative;
		height: 3.6rem;
		min-width: 12rem;
		display: flex;
		justify-content: center;
		padding: 1rem 2rem;
		background: hsla(0deg, 0%, 40%, 20%);
		backdrop-filter: blur(2rem);
		border-radius: 0.5rem;
		overflow: clip;

		.paint-inner {
			position: absolute;
		}
	}
</style>
