<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLLabelAttributes } from "svelte/elements";

	type Props = {
		option?: boolean;
		name: string;
		value: string;
		group?: string;
		disabled?: boolean;
		onclick?: (e: MouseEvent) => void;
		leftLabel?: Snippet;
		children?: Snippet;
	} & HTMLLabelAttributes;

	let {
		option = false,
		name,
		value,
		group = $bindable(),
		disabled = false,
		onclick,
		leftLabel,
		children,
		...restProps
	}: Props = $props();
</script>

<label class:option {...restProps}>
	{@render leftLabel?.()}
	<input type="radio" bind:group {value} {name} {disabled} {onclick} />
	<span class="checkbox"></span>
	{@render children?.()}
</label>

<style lang="scss">
	label {
		user-select: none;

		display: flex;
		gap: 0.75rem;

		color: var(--text);
		font-size: 0.875rem;

		&:has(input:enabled) {
			cursor: pointer;

			&.option {
				&:focus-visible,
				&:hover {
					border-color: var(--border-active);
				}
			}
		}

		&:has(input:checked) {
			// I hate CSS
			&:has(input:enabled),
			&:has(input:disabled) {
				&.option {
					border-color: var(--primary);
				}
			}
		}

		&.option {
			padding: 0.97rem 0.75rem;
			border-radius: 0.5rem;
			background-color: var(--bg-medium);
			border: 1px solid transparent;

			justify-content: space-between;
			align-items: center;
		}

		.checkbox {
			flex-shrink: 0;

			display: inline-block;
			height: 1rem;
			width: 1rem;

			border: 1px solid var(--secondary-border);
			border-radius: 50%;

			background-color: var(--secondary);
			transition: background 0.1s;

			position: relative;

			&::after {
				content: "";

				position: absolute;
				top: 0;
				left: 0;
				right: 0;
				bottom: 0;
				margin: auto;

				height: 0.5rem;
				width: 0.5rem;
				border-radius: 50%;
				background-color: var(--secondary);

				transform: scale(0);
				transition: transform 0.1s;
			}
		}

		input {
			position: absolute;
			margin: 0;
			width: 0;
			height: 0;
			display: inline;
			-webkit-appearance: none;
			-moz-appearance: none;
			appearance: none;
			outline: none;
		}

		input:disabled + .checkbox {
			cursor: not-allowed;
			color: var(--text-light);
			background-color: var(--secondary-disabled);
		}

		input:enabled {
			&:focus-visible + .checkbox {
				border-color: var(--primary);
			}

			&:focus-visible,
			&:hover {
				& + .checkbox {
					background-color: var(--secondary-hover);
				}
			}

			&:active + .checkbox {
				background-color: var(--secondary-active);
			}
		}

		input:enabled,
		input:disabled {
			&:checked + .checkbox {
				background-color: var(--primary);
				border-color: transparent;

				&::after {
					transform: scale(1);
				}
			}
		}
	}
</style>
