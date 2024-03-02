<script lang="ts">
	export let value: boolean = false;
    export let disabled: boolean = false;
	export let numb: boolean = false;
</script>

<label class="switch" class:numb={numb} {...$$restProps}>
	<input type="checkbox" disabled={disabled} bind:checked={value} />
	<span class="slider"></span>
</label>

<style lang="scss">
	/* The switch - the box around the slider */
	.switch {
		position: relative;
		display: inline-block;
		width: 2.25rem;
		height: 1.25rem;

		&.numb {
			pointer-events: none;
		}
	}

	/* Hide default HTML checkbox */
	.switch input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	/* The slider */
	.slider {
        cursor: pointer;
		background-color: var(--border-active);
        border-radius: 0.75rem;

		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;

		transition: background-color 0.2s;
	}

	.slider:before {
		content: "";
		height: 1rem;
		width: 1rem;
        border-radius: 50%;

		position: absolute;
		left: 0.125rem;
		bottom: 0.125rem;

		background-color: var(--primary-text);
		transition: transform 0.2s;
	}

	input:checked + .slider {
		background-color: var(--primary);
	}

    input:disabled + .slider {
        cursor: not-allowed;
    }

	input:checked:disabled + .slider {
		background-color: var(--primary-disabled);
	}

	input:checked + .slider:before {
		transform: translateX(1rem);
	}

    input:disabled + .slider::before {
        background-color: var(--text-light);
    }
</style>
