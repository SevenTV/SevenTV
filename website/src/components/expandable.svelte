<script lang="ts">
	import { faChevronDown } from "@fortawesome/pro-solid-svg-icons";
	import { CaretDown } from "phosphor-svelte";
	import Fa from "svelte-fa";

	export let title: string;

	let expanded = true;
</script>

<div class="expandable">
	<button class="header" on:click={() => (expanded = !expanded)} class:expanded>
		{title}
		<div class="icon">
			<CaretDown size="1rem" />
		</div>
	</button>
	{#if expanded}
		<slot />
	{/if}
</div>

<style lang="scss">
	.expandable {
		display: flex;
		flex-direction: column;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;

		color: var(--text-lighter);
		font-size: 0.75rem;
		font-weight: 500;

		& > .icon {
			transition: transform 0.1s;
			transform: rotate(-90deg);
		}

		&.expanded {
			margin-bottom: 0.75rem;

			& > .icon {
				transform: rotate(0deg);
			}
		}
	}
</style>
