<script context="module">
	let dropDownIndex = 0;
</script>

<script lang="ts">
	import mouseTrap from "$/lib/mouseTrap";

	let index = dropDownIndex;
	dropDownIndex += 1;

	let expanded = false;

	function toggle() {
		expanded = !expanded;
	}

	function close() {
		expanded = false;
	}
</script>

<button
	on:click={toggle}
	aria-expanded={expanded}
	aria-controls="dropdown-list-{index}"
	use:mouseTrap={close}
>
	<slot />
	{#if expanded}
		<ul class="list" id="dropdown-list-{index}">
			<slot name="dropdown" />
		</ul>
	{/if}
</button>

<style lang="scss">
	button {
		position: relative;

		.list {
			display: flex;
			flex-direction: column;

			z-index: 1;

			position: absolute;
			right: 0;
			margin: 0;
			padding: 0;
			border: var(--border) 1px solid;
            border-radius: 0.25rem;

			background-color: var(--bg-medium);
			// filter: drop-shadow(0 0 0.25rem rgba(0, 0, 0, 0.25));
            box-shadow: 4px 4px 0px rgba(0, 0, 0, 0.25);

			:global(li) {
				display: flex;
				flex-direction: column;
				align-items: stretch;

				&:hover,
				&:focus-visible {
					background-color: var(--bg-light);
				}
			}

            :global(hr) {
                width: 80%;
                height: 1px;
                align-self: center;
            }

			:global(a),
			:global(button) {
				padding: 0.75rem;

				text-align: left;
				color: var(--text);
                font-size: 0.8125rem;
				text-decoration: none;
				font-weight: 500;
                white-space: nowrap;

				display: flex;
				align-items: center;
				gap: 0.5rem;
			}
		}
	}
</style>
