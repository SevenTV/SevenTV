<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLLabelAttributes } from "svelte/elements";

	type Props = {
		value?: string;
		type?: "text" | "email" | "password" | "textarea";
		placeholder?: string | null;
		big?: boolean;
		required?: boolean;
		minlength?: number;
		maxlength?: number;
		disabled?: boolean;
		children?: Snippet;
		nonLabelChildren?: Snippet;
		icon?: Snippet;
		onkeypress?: (e: KeyboardEvent) => void;
	} & HTMLLabelAttributes;

	let {
		value = $bindable(),
		type = "text",
		placeholder = null,
		big = false,
		required = false,
		minlength,
		maxlength,
		disabled = false,
		children,
		nonLabelChildren,
		icon,
		onkeypress,
		...restProps
	}: Props = $props();

	// svelte-ignore non_reactive_update
	let input: HTMLElement;

	export function focus() {
		input?.focus();
	}

	export function blur() {
		input?.blur();
	}
</script>

<label class="input" class:big class:has-label={children} class:has-icon={icon} {...restProps}>
	{@render nonLabelChildren?.()}
	{@render children?.()}
	{#if icon}
		<div class="icon">
			{@render icon()}
		</div>
	{/if}
	{#if type === "text"}
		<input
			type="text"
			bind:value
			{placeholder}
			{onkeypress}
			{minlength}
			{maxlength}
			{disabled}
			{required}
			bind:this={input}
		/>
	{:else if type === "email"}
		<input
			type="email"
			bind:value
			{placeholder}
			{onkeypress}
			{minlength}
			{maxlength}
			{disabled}
			{required}
			bind:this={input}
		/>
	{:else if type === "password"}
		<input
			type="password"
			bind:value
			{placeholder}
			{onkeypress}
			{minlength}
			{maxlength}
			{disabled}
			{required}
			bind:this={input}
		/>
	{:else if type === "textarea"}
		<textarea
			bind:value
			{placeholder}
			{onkeypress}
			{minlength}
			{maxlength}
			{disabled}
			{required}
			bind:this={input}
		></textarea>
	{/if}
</label>

<style lang="scss">
	.input {
		--icon-left-padding: 0.75rem;
		--gap: 0.5rem;
		--input-block-padding: 0.5rem;

		position: relative;

		&.big {
			--icon-left-padding: 1rem;
			--gap: 0.75rem;
			--input-block-padding: 0.6rem;
		}

		input,
		textarea {
			padding-block: var(--input-block-padding);
		}

		textarea {
			height: 7lh;
		}

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
