<script lang="ts">
	import type { Role } from "$/gql/graphql";

	let { role }: { role: Role } = $props();
</script>

<span style:color={role.color?.hex}>{role.name}</span>

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
