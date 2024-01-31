<script lang="ts">
	import Fa from "svelte-fa";
	import { faSearch } from "@fortawesome/pro-solid-svg-icons";
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
			padding: 0.75rem 0;
			padding-left: 2.5rem;
		}
	}
</style>
