<script lang="ts">
	import type { Badge, Paint } from "$/gql/graphql";
	import BadgeComponent from "../badge.svelte";
	import ResponsiveImage from "../responsive-image.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";

	interface Props {
		mode: DialogMode;
		badge: Badge;
	}

	let { mode = $bindable("hidden"), badge }: Props = $props();
</script>

<Dialog width={30} bind:mode>
	<div class="layout">
		<h1>Badge</h1>
		<hr />
		<div class="badge-container">
			<ResponsiveImage images={badge.images} width={5 * 16} title={badge.description} />
			<h2>{badge.name}</h2>
		</div>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		height: 100%;
	}

	.badge-container {
		margin: auto;
		padding: 2rem;

		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}
</style>
