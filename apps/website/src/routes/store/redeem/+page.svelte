<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { CaretRight, Check, Gift } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { user } from "$lib/auth";
	import Banner from "$/components/store/banner.svelte";
	import StoreSection from "$/components/store/store-section.svelte";
	import type { PageData } from "./$types";
	import { signInDialogMode } from "$/lib/layout";
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";

	let { data }: { data: PageData } = $props();

	let code = $state(data.code ?? "");
	let redeemState = $state<"idle" | "loading" | "success">("idle");

	$effect(() => {
		let url = new URL($page.url);

		if (code) {
			url.searchParams.set("code", code);
		} else {
			url.searchParams.delete("code");
		}

		goto(url, { replaceState: true, noScroll: true, keepFocus: true });
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (!$user) {
			$signInDialogMode = "shown";
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

<svelte:head>
	<title>Redeem - {$t("page_titles.suffix")}</title>
</svelte:head>

<Banner
	title="Redeem a Gift Code"
	subtitle="Redeem a gift code to unlock exclusive cosmetics and benefits."
	gradientColor="#ff11bc"
/>

<div class="grid">
	<StoreSection title="Redeem a Gift Code">
		<form class="redeem" onsubmit={submit}>
			<TextInput
				placeholder={$t("labels.redeem")}
				style="flex-grow: 1"
				disabled={redeemState !== "idle"}
				bind:value={code}
			>
				<span class="label">Code</span>
				{#snippet icon()}
					<Gift />
				{/snippet}
			</TextInput>
			<Button secondary submit disabled={redeemState !== "idle" || !code} style="align-self: end">
				Redeem
				{#snippet iconRight()}
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
	</StoreSection>
</div>

<style lang="scss">
	.label {
		font-size: 0.75rem;
		font-weight: 500;
	}

	.grid {
		max-width: 25rem;
		margin-top: 1rem;
		margin-inline: auto;
	}

	.redeem {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		& :global(input) {
			// Monospace for redeem codes
			font-family: monospace, "Inter", sans-serif;
		}
	}
</style>
