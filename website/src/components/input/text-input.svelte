<script lang="ts">
	export let value = "";
	export let placeholder: string | null = null;
</script>

<div class="input" class:has-icon={$$slots.default} {...$$restProps}>
	{#if $$slots.default}
		<div class="icon">
			<slot />
		</div>
	{/if}
	<input type="text" bind:value={value} {placeholder} on:keypress />
</div>

<style lang="scss">
	.input {
		--icon-left-padding: 0.75rem;
		--gap: 0.5rem;

		position: relative;

		display: flex;
		align-items: center;
		justify-content: center;

		.icon {
			position: absolute;
			top: 0;
			left: 0;
			bottom: 0;

			display: flex;
			align-items: center;
			padding-left: var(--icon-left-padding);
			color: var(--text-light);
			pointer-events: none;
		}

		&.has-icon > input {
			// icon left padding + icon width + gap
			padding-left: calc(var(--icon-left-padding) + 1rem + var(--gap));
		}
	}
</style>
