<script lang="ts">
	import { Warning } from "phosphor-svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		cancel?: () => void;
		confirm?: () => void;
		logInLast?: () => void;
		lastPlatform: string;
	}

	let { mode = $bindable("hidden"), cancel, confirm, logInLast, lastPlatform }: Props = $props();

	const confirmed = () => {
		confirm?.();
		mode = "hidden";
	};

	const loginLastPlatform = () => {
		logInLast?.();
		mode = "hidden";
	};

	const canceled = () => {
		cancel?.();
		mode = "hidden";
	};
</script>

<Dialog width={40} bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.delete_account.warning")}</h1>
		<hr />
		<section id="warning">
			<Warning size="5rem" color="var(--danger)" />
			<p id="warning-text">Are you sure you want to proceed?</p>
			<p>We detected a previous login with {lastPlatform}.</p>
			<p>Choose "Login with {lastPlatform}" to start the linking process for that account.</p>
			<p>Creating a new account will make a separate account.</p>
			<a href="https://help.7tv.app/en/articles/11068377" target="_blank">
				Learn how to connect accounts
			</a>
		</section>
		<div class="buttons">
			<Button primary onclick={loginLastPlatform}>Login with {lastPlatform}</Button>
			<Button style="color: var(--danger)" onclick={confirmed}>Create New Account</Button>
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

		#warning-text {
			font-size: 1.5rem;
			font-weight: 600;
			color: var(--danger);
		}

		#warning {
			display: flex;
			flex-direction: column;
			align-items: center;
			text-align: center;
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

		@media (max-width: 768px) {
			flex-direction: column;
			justify-content: stretch;
		}
	}
</style>
