<script lang="ts">
	export let value = "";
	export let type: "text" | "email" | "password" | "textarea" = "text";
	export let placeholder: string | null = null;

	export let big: boolean = false;
</script>

<label
	class="input"
	class:big
	class:has-label={$$slots.default}
	class:has-icon={$$slots.icon}
	{...$$restProps}
>
	<slot />
	{#if $$slots.icon}
		<div class="icon">
			<slot name="icon" />
		</div>
	{/if}
	{#if type === "text"}
		<input type="text" bind:value {placeholder} on:keypress />
	{:else if type === "email"}
		<input type="email" bind:value {placeholder} on:keypress />
	{:else if type === "password"}
		<input type="password" bind:value {placeholder} on:keypress />
	{:else if type === "textarea"}
		<textarea bind:value {placeholder} on:keypress />
	{/if}
</label>

<style lang="scss">
	.input {
		--icon-left-padding: 0.75rem;
		--gap: 0.5rem;
		--input-block-padding: 0.5rem;

		&.big {
			--icon-left-padding: 1rem;
			--gap: 0.75rem;
			--input-block-padding: 0.75rem;
		}

		input, textarea {
			padding-block: var(--input-block-padding);
		}

		textarea {
			height: 7lh;
		}

		position: relative;

		.icon {
			position: absolute;
			left: 0;
			bottom: var(--input-block-padding);

			display: flex;
			align-items: center;
			padding-left: var(--icon-left-padding);
			color: var(--text-light);
			pointer-events: none;
		}

		&.has-label > input {
			margin-top: 0.25rem;
		}

		&.has-icon > input {
			// icon left padding + icon width + gap
			padding-left: calc(var(--icon-left-padding) + 1rem + var(--gap));
		}
	}
</style>
