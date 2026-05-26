<script lang="ts">
	import { Warning } from "phosphor-svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		cancel?: () => void;
		confirm?: () => void;
	}

	let { mode = $bindable("hidden"), cancel, confirm }: Props = $props();

	const confirmed = () => {
		confirm?.();
		mode = "hidden";
	};

	const canceled = () => {
		cancel?.();
		mode = "hidden";
	};
</script>

<Dialog bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.delete_account.warning")}</h1>
		<hr />
		<section id="warning">
			<Warning size="5rem" color="var(--danger)" />
			Are you sure you want to proceed?
			<small>You cannot undo this action.</small>
		</section>
		<div class="buttons">
			<Button style="color: var(--danger)" onclick={confirmed}>{$t("labels.proceed")}</Button>
			<Button secondary onclick={canceled}>{$t("labels.cancel")}</Button>
		</div>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		#warning {
			display: flex;
			flex-direction: column;
			align-items: center;
		}

		height: 100%;

		z-index: 100000000;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
