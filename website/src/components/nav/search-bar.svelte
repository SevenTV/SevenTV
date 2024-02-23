<script lang="ts">
	import { MagnifyingGlass } from "phosphor-svelte";

	export let big: boolean = false;
</script>

<search class:big>
	<div class="icon">
		<MagnifyingGlass />
	</div>
	<input type="text" placeholder="Search" />
</search>

<style lang="scss">
	search {
		--icon-left-padding: 0.75rem;
		--gap: 0.5rem;

		&.big {
			--icon-left-padding: 1rem;
			--gap: 0.75rem;

			/* First, take 30rem and then shrink by a factor of 1 */
			flex: 0 1 30rem;

			input {
				padding-block: 0.75rem;
			}
		}

		&:not(.big) {
			max-width: 12.5rem;
		}

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

		input {
			// icon left padding + icon width + gap
			padding-left: calc(var(--icon-left-padding) + 1rem + var(--gap));
		}
	}
</style>
