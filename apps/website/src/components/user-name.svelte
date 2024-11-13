<script lang="ts">
	import type { User } from "$/gql/graphql";
	import Paint from "./paint.svelte";

	let { user }: { user: User } = $props();
</script>

{#if user.style.activePaint}
	<Paint paint={user.style.activePaint.data}>{user.mainConnection?.platformDisplayName}</Paint>
{:else}
	{user.mainConnection?.platformDisplayName}
{/if}
