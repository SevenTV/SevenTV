<script lang="ts">
	import type { Role } from "$/gql/graphql";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = {
		roleData: Role;
	} & HTMLAttributes<HTMLSpanElement>;

	let { roleData, ...otherProps }: Props = $props();
</script>

<span {...otherProps} style:color={roleData.color?.hex}>{roleData.name}</span>

<style lang="scss">
	span {
		padding: 0.2rem 0.5rem;

		color: var(--color);
		font-size: 0.75rem;
		font-weight: 500;

		position: relative;

		&::before {
			content: "";
			position: absolute;
			background-color: currentColor;
			border-radius: 0.5rem;
			opacity: 0.1;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
		}
	}
</style>
