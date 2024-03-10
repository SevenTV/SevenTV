<script lang="ts">
	export let option: boolean = false;
	export let name: string;

	export let value = false;
	export let disabled = false;
</script>

<label class:option {...$$restProps}>
	<slot name="left-label" />
	<input type="radio" bind:group={value} {name} {disabled} on:click />
	<span class="checkbox"></span>
	<slot />
</label>

<style lang="scss">
	label {
		cursor: pointer;
		user-select: none;

		display: flex;
		gap: 0.75rem;

		color: var(--text);
		font-size: 0.875rem;

		&.option {
			padding: 0.97rem 0.75rem;
			border-radius: 0.5rem;
			background-color: var(--bg-medium);
			border: 1px solid transparent;

			justify-content: space-between;
			align-items: center;

			&:has(input:checked) {
				border-color: var(--primary);
			}

			&:focus-visible,
			&:hover {
				border-color: var(--border-active);
			}
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
				background-color: white;

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

		input:focus-visible + .checkbox {
			border-color: var(--primary);
		}

		input:focus-visible,
		input:hover {
			& + .checkbox {
				background-color: var(--secondary-hover);
			}
		}

		input:active + .checkbox {
			background-color: var(--secondary-active);
		}

		input:checked + .checkbox {
			background-color: var(--primary);
			border-color: transparent;

			&::after {
				transform: scale(1);
			}
		}

		input:disabled {
			cursor: not-allowed;
			color: var(--text-light);

			& + .checkbox {
				background-color: var(--secondary-disabled);
			}
		}
	}
</style>
