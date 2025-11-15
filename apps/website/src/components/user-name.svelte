<script lang="ts">
	import type { User } from "$/gql/graphql";
	import type { HTMLAttributes } from "svelte/elements";
	import Paint from "./paint.svelte";

	type Props = {
		user: User;
		enablePaintDialog?: boolean;
	} & HTMLAttributes<HTMLSpanElement>;

	let { user, enablePaintDialog, ...restProps }: Props = $props();
</script>

{#snippet name()}
	<span
		class="name"
		title={user.mainConnection?.platformDisplayName}
		style:color={user.highestRoleColor?.hex}
		{...restProps}
	>
		{user.mainConnection?.platformDisplayName}
	</span>
{/snippet}

{#if user.style.activePaint}
	<Paint paint={user.style.activePaint} enableDialog={enablePaintDialog} style="display: inline; color: {user.highestRoleColor?.hex || null};">
		{@render name()}
	</Paint>
{:else}
	{@render name()}
{/if}

<style lang="scss">
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
