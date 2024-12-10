<script lang="ts">
	import type { Badge } from "$/gql/graphql";
	import type { HTMLAttributes } from "svelte/elements";
	import BadgeDialog from "./dialogs/badge-dialog.svelte";
	import type { DialogMode } from "./dialogs/dialog.svelte";
	import ResponsiveImage from "./responsive-image.svelte";

	type Props = {
		badge: Badge;
		size?: number;
		enableDialog?: boolean;
	} & HTMLAttributes<HTMLButtonElement> &
		HTMLAttributes<HTMLDivElement>;

	let { badge, size = 1.25 * 16, enableDialog = false, ...restProps }: Props = $props();

	let dialogMode: DialogMode = $state("hidden");

	function showDialog() {
		dialogMode = "shown";
	}
</script>

{#if enableDialog}
	<BadgeDialog bind:mode={dialogMode} {badge} />
	<button onclick={showDialog} style="display: flex;" title={badge.description} {...restProps}>
		<ResponsiveImage images={badge.images} width={size} height={size} />
	</button>
{:else}
	<div style="display: flex;" title={badge.description} {...restProps}>
		<ResponsiveImage images={badge.images} width={size} height={size} />
	</div>
{/if}
