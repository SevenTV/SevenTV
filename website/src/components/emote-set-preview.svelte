<script lang="ts">
	export let name = "Personal Emotes";
	export let percentage = 0;
	export let bg: "medium" | "light" = "medium";
</script>

<a href="/emote-set/{name}" class="emote-set" style:background-color="var(--bg-{bg})">
	<div class="emotes">
		{#each Array(12) as _}
			<div class="emote"></div>
		{/each}
	</div>
	<span class="name">{name}</span>
	<div class="percentage">
		{percentage}%
	</div>
</a>

<style lang="scss">
	.emote-set {
		color: var(--text);
		text-decoration: none;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		padding: 1rem;
		border: 1px solid transparent;
		border-radius: 0.25rem;
		cursor: pointer;

		&:hover,
		&:focus-visible {
			border-color: var(--border);
		}
	}

	.emotes {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 0.5rem;
	}

	.emote {
		width: 2rem;
		height: 2rem;
		background-color: var(--primary);
		border-radius: 0.25rem;
	}

	.name {
		margin-top: 0.5rem;
		color: #aa71ff;
		font-size: 0.875rem;
		font-weight: 600;
	}

	.percentage {
		padding: 0.25rem;
		text-align: center;
		font-size: 0.75rem;
		font-weight: 500;

		background-color: var(--primary);
		border-radius: 0.25rem;
	}
</style>
