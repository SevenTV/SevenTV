<script lang="ts" context="module">
	let checkboxIndex = 0;
</script>

<script lang="ts">
	export let label: string = "";
	export let value = false;
	export let disabled = false;

	const id = checkboxIndex++;
</script>

<div class="checkbox" {...$$restProps}>
	<input type="checkbox" id="checkbox-{id}" bind:checked={value} {disabled} />
	<label for="checkbox-{id}">{label}</label>
</div>

<style lang="scss">
	.checkbox {
		overflow: hidden;

		label {
			user-select: none;
			cursor: pointer;
			display: flex;
			align-items: center;
			gap: 0.75rem;
			font-size: 0.8125rem;
			font-weight: 400;
			color: var(--text);

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

		input:focus-visible + label::before {
			border-color: var(--primary);
		}

		input:focus-visible,
		input:hover {
			& + label::before {
				background-color: var(--secondary-hover);
			}
		}

		input:active + label::before {
			background-color: var(--secondary-active);
		}

		input:checked + label::before {
			background-color: var(--primary);
			background-size: 0.75rem;
			border-color: transparent;
		}

		input:disabled + label {
			cursor: not-allowed;
			color: var(--text-light);

			&::before {
				background-color: var(--secondary-disabled);
			}
		}
	}
</style>
