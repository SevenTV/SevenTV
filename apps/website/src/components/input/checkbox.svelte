<script lang="ts">
	export let option: boolean = false;

	export let value = false;
	export let disabled = false;
	export let indeterminate: boolean = false;
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<label class:option class:disabled {...$$restProps} on:click|stopPropagation>
	<slot name="left-label" />
	<input type="checkbox" bind:checked={value} bind:indeterminate on:click {disabled} />
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

		&.disabled {
			cursor: not-allowed;
		}

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
			border-radius: 0.25rem;

			background-color: var(--secondary);
			background-size: 0;
			background-repeat: no-repeat;
			background-position: center;

			transition: background 0.1s;
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
			// Checked image
			background-image: url('data:image/svg+xml;charset=UTF-8,<svg width="8" height="8" viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M7.26531 2.51531L3.26531 6.51531C3.23048 6.55027 3.18908 6.57801 3.1435 6.59694C3.09791 6.61587 3.04904 6.62561 2.99969 6.62561C2.95033 6.62561 2.90146 6.61587 2.85588 6.59694C2.8103 6.57801 2.7689 6.55027 2.73406 6.51531L0.984064 4.76531C0.949182 4.73043 0.921512 4.68902 0.902633 4.64344C0.883755 4.59787 0.874039 4.54902 0.874039 4.49969C0.874039 4.45036 0.883755 4.40151 0.902633 4.35593C0.921512 4.31036 0.949182 4.26895 0.984064 4.23406C1.01895 4.19918 1.06036 4.17151 1.10593 4.15263C1.15151 4.13375 1.20036 4.12404 1.24969 4.12404C1.29902 4.12404 1.34787 4.13375 1.39344 4.15263C1.43902 4.17151 1.48043 4.19918 1.51531 4.23406L3 5.71875L6.73469 1.98469C6.80514 1.91424 6.90069 1.87466 7.00031 1.87466C7.09994 1.87466 7.19549 1.91424 7.26594 1.98469C7.33639 2.05514 7.37597 2.15068 7.37597 2.25031C7.37597 2.34994 7.33639 2.44549 7.26594 2.51594L7.26531 2.51531Z" fill="black"/></svg>');
			background-color: white;
			background-size: 1rem;
		}

		input:indeterminate + .checkbox {
			// Indeterminate image
			background-image: url('data:image/svg+xml;charset=UTF-8,<svg width="8" height="8" viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M7.125 4C7.125 4.09946 7.08549 4.19484 7.01516 4.26516C6.94484 4.33549 6.84946 4.375 6.75 4.375H1.25C1.15054 4.375 1.05516 4.33549 0.984835 4.26516C0.914509 4.19484 0.875 4.09946 0.875 4C0.875 3.90054 0.914509 3.80516 0.984835 3.73483C1.05516 3.66451 1.15054 3.625 1.25 3.625H6.75C6.84946 3.625 6.94484 3.66451 7.01516 3.73483C7.08549 3.80516 7.125 3.90054 7.125 4Z" fill="black"/></svg>');
			background-color: white;
			background-size: 1rem;
		}

		input:disabled {
			color: var(--text-light);

			& + .checkbox {
				background-color: var(--secondary-disabled);
			}

			&:checked + .checkbox {
				background-color: gray;
			}

			&:indeterminate + .checkbox {
				background-color: gray;
			}
		}
	}
</style>
