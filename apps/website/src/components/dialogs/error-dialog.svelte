<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		error?: string;
	}

	let { mode = $bindable("hidden"), error = "An error occured" }: Props = $props();
</script>

<Dialog width={30} bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.error.title")}</h1>
		<hr />
		<p>{error}</p>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}
</style>
