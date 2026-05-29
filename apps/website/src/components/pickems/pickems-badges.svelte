<script lang="ts">
	import type { Badge } from "$/gql/graphql";
	import BadgeComponent from "../badge.svelte";
	import { t } from "svelte-i18n";

	let { badges }: { badges: Badge[] } = $props();
</script>

<div class="badges">
	{#each badges as badge}
		<BadgeComponent enableDialog size={48} {badge} />
	{/each}
</div>

<style lang="scss">
	.badges {
		display: flex;
		gap: 1rem;
		padding: 0.5rem;
		background: hsla(0deg, 0%, 40%, 20%);
		backdrop-filter: blur(2rem);
		border-radius: 0.5rem;
	}
</style>
