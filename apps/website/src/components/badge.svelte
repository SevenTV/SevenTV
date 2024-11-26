<script lang="ts">
	import type { Badge } from "$/gql/graphql";
	import BadgeDialog from "./dialogs/badge-dialog.svelte";
	import type { DialogMode } from "./dialogs/dialog.svelte";
	import ResponsiveImage from "./responsive-image.svelte";

	let { badge, size = 1.25 * 16 }: { badge: Badge; size?: number } = $props();

	let dialogMode: DialogMode = $state("hidden");

	function showDialog() {
		dialogMode = "shown";
	}
</script>

<BadgeDialog bind:mode={dialogMode} {badge} />
<button onclick={showDialog} style="display: flex;" title={badge.description}>
	<ResponsiveImage images={badge.images} height={size} />
</button>
