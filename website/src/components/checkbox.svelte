<script lang="ts">
	export let value = false;
	export let disabled = false;
</script>

<label class="checkbox" {...$$restProps}>
	<input type="checkbox" bind:checked={value} {disabled} />
	<slot />
</label>

<style lang="scss">
	.checkbox {
		cursor: pointer;
		user-select: none;

		display: flex;
		gap: 0.75rem;

		color: var(--text);
		font-size: 0.875rem;

		&::before {
			content: "";
			display: inline-block;
			height: 1rem;
			width: 1rem;

			border: 1px solid var(--secondary-border);
			border-radius: 0.25rem;

			background-color: var(--secondary);
			background-image: url('data:image/svg+xml;charset=UTF-8,<svg width="11" height="8" viewBox="0 0 11 8" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M10.2797 0.470215C10.5727 0.763184 10.5727 1.23896 10.2797 1.53193L4.27974 7.53193C3.98677 7.8249 3.51099 7.8249 3.21802 7.53193L0.218018 4.53193C-0.0749512 4.23897 -0.0749512 3.76318 0.218018 3.47021C0.510986 3.17725 0.986768 3.17725 1.27974 3.47021L3.75005 5.93818L9.22036 0.470215C9.51333 0.177246 9.98911 0.177246 10.2821 0.470215H10.2797Z" fill="white"/></svg>');
			background-size: 0;
			background-repeat: no-repeat;
			background-position: 2px center;

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

		&:has(input:focus-visible)::before {
			border-color: var(--primary);
		}

		&:has(input:focus-visible)::before, &:has(input:hover)::before {
			background-color: var(--secondary-hover);
		}

		&:has(input:active)::before {
			background-color: var(--secondary-active);
		}

		&:has(input:checked)::before {
			background-color: var(--primary);
			background-size: 0.75rem;
			border-color: transparent;
		}

		&:has(input:disabled) {
			cursor: not-allowed;
			color: var(--text-light);

			&::before {
				background-color: var(--secondary-disabled);
			}
		}
	}
</style>
