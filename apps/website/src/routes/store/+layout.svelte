<script lang="ts">
	import { CaretRight, Check, Gift, PaintBrush, Star } from "phosphor-svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import Button from "$/components/input/button.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { user } from "$/lib/auth";

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();

	let code = $state("");
	let redeemState = $state<"idle" | "loading" | "success">("idle");

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (!$user) {
			return;
		}

		redeemState = "loading";

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation RedeemCode($userId: Id!, $code: String!) {
						billing(userId: $userId) {
							redeemCode(code: $code) {
								checkoutUrl
							}
						}
					}
				`),
				{ userId: $user.id, code },
			)
			.toPromise();

		if (res.data) {
			if (res.data.billing.redeemCode.checkoutUrl) {
				window.location.href = res.data.billing.redeemCode.checkoutUrl;
			}

			redeemState = "success";
		} else {
			redeemState = "idle";
		}
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.store.title")}</h1>
		<nav class="link-list">
			<TabLink href="/store" title={$t("common.subscriptions", { values: { count: 1 } })} big>
				<Star />
				{#snippet active()}
					<Star weight="fill" />
				{/snippet}
			</TabLink>
			<TabLink href="/store/paint-bundles" title={$t("common.paint_bundles")} big>
				<PaintBrush />
				{#snippet active()}
					<PaintBrush weight="fill" />
				{/snippet}
			</TabLink>
		</nav>
		<hr />
		<form class="redeem" onsubmit={submit}>
			<TextInput
				placeholder={$t("labels.redeem")}
				style="flex-grow: 1"
				disabled={redeemState !== "idle"}
				bind:value={code}
			>
				{#snippet icon()}
					<Gift />
				{/snippet}
				<span class="label">{$t("pages.store.redeem")}</span>
			</TextInput>
			<Button secondary submit disabled={redeemState !== "idle" || !code}>
				{#snippet icon()}
					{#if redeemState === "idle"}
						<CaretRight />
					{:else if redeemState === "loading"}
						<Spinner />
					{:else if redeemState === "success"}
						<Check />
					{/if}
				{/snippet}
			</Button>
		</form>
	</aside>
	<div class="content">
		{@render children()}
	</div>
</div>

<style lang="scss">
	.redeem {
		display: flex;
		align-items: end;
		gap: 0.5rem;

		& :global(input) {
			// Monospace for redeem codes
			font-family: monospace, "Inter", sans-serif;
		}
	}

	.label {
		color: var(--text-light);
		font-size: 0.75rem;
		font-weight: 500;
	}

	// Only desktop
	@media screen and (min-width: 961px) {
		.content {
			overflow: auto;
			scrollbar-gutter: stable;

			& > :global(*) {
				// right margin because of side-bar-layout
				margin-right: 1.25rem;
			}
		}
	}
</style>
