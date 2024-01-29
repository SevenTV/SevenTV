<script lang="ts">
	import Fa from "svelte-fa";
	import { faSearch } from "@fortawesome/free-solid-svg-icons";
</script>

<search>
	<div class="icon">
		<Fa icon={faSearch} />
	</div>
	<input type="text" placeholder="Search" />
</search>

<style lang="scss">
	search {
		/* First, take 30rem and then shrink by a factor of 1 */
		flex: 0 1 30rem;

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
			padding-left: 1rem;
			color: var(--text-light);
			pointer-events: none;
		}

		input {
			width: 100%;

			padding: 0.75rem 0;
			padding-left: 2.5rem;
			border-radius: 0.5rem;
			border: 1px solid transparent;
			outline: none;
			background-color: var(--secondary);
			color: var(--text);
			font-size: 0.875rem;

			transition: background-color 0.1s;

			&:focus-visible {
				background-color: var(--secondary-hover);
				border: 1px solid var(--primary);
			}

			&:active {
				background-color: var(--secondary-active);
			}
		}
	}
</style>
