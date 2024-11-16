<script lang="ts">
	import type { User } from "$/gql/graphql";
	import Paint from "./paint.svelte";

	let { user }: { user: User } = $props();
</script>

{#snippet name()}
	<span class="name" style:color={user.highestRoleColor?.hex}
		>{user.mainConnection?.platformDisplayName}</span
	>
{/snippet}

{#if user.style.activePaint}
	<Paint paint={user.style.activePaint}>
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
