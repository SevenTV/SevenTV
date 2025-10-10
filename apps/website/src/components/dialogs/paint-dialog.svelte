<script lang="ts">
	import type { Paint } from "$/gql/graphql";
	import PaintComponent from "../paint.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		paint: Paint;
	}

	let { mode = $bindable("hidden"), paint }: Props = $props();
</script>

<Dialog width={30} bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.paint.title")}</h1>
		<hr />
		<div class="paint-container">
			<PaintComponent {paint}>
				{paint.name}
			</PaintComponent>
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

	.paint-container {
		margin: auto;
		padding: 2rem;

		font-size: 3rem;
		font-weight: 700;
		text-align: center;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}
</style>
