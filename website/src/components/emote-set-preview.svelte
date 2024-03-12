<script lang="ts">
	import Flags, { determineHighlightColor } from "./flags.svelte";

	export let name = "Personal Emotes";
	export let percentage = 0;
	export let bg: "medium" | "light" = "medium";

	let flags = ["trending", "personal"];

	$: highlight = determineHighlightColor(flags);
</script>

<a
	href="/emote-sets/{name}"
	class="emote-set"
	style:border-color="{highlight}80"
	style:background-color="var(--bg-{bg})"
>
	<div class="emotes">
		{#each Array(12) as _}
			<div class="emote"></div>
		{/each}
	</div>
	<span class="name">{name}</span>
	<Flags {flags} iconOnly style="justify-content: flex-end" />
	<div class="percentage">
		{percentage}%
	</div>
</a>

<style lang="scss">
	.emote-set {
		color: var(--text);
		text-decoration: none;

		display: grid;
		gap: 0.5rem;

		padding: 1rem;
		border: 1px solid transparent;
		border-radius: 0.25rem;
		cursor: pointer;

		&:hover,
		&:focus-visible {
			border-color: var(--border-active);
		}
	}

	.emotes {
		grid-column: span 2;
		margin-bottom: 0.5rem;

		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 0.5rem;

		.emote {
			width: 2rem;
			height: 2rem;
			background-color: var(--secondary);
			border-radius: 0.25rem;
		}
	}

	.name {
		color: var(--subscriber);
		font-size: 0.875rem;
		font-weight: 600;
	}

	.percentage {
		grid-column: span 2;

		padding: 0.25rem;
		text-align: center;
		font-size: 0.75rem;
		font-weight: 500;

		background-color: var(--secondary);
		border-radius: 0.25rem;
	}
</style>
