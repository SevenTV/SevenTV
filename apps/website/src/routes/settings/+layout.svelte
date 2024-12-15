<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { user } from "$/lib/auth";
	import { Key, PencilSimple, CreditCard, ArrowSquareOut, PaintBrushBroad } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import UserName from "$/components/user-name.svelte";
	import UserProfilePicture from "$/components/user-profile-picture.svelte";
	import { PUBLIC_STRIPE_CUSTOMER_PORTAL } from "$env/static/public";
	import NumberBadge from "$/components/number-badge.svelte";
	import { SubscriptionProvider } from "$/gql/graphql";
	import { pendingEditorFor } from "$/lib/auth";

	let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
	<title>{$t("common.settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("common.settings")}</h1>
		<nav class="link-list">
			{#if $user}
				<TabLink title={$t("pages.settings.account.title")} href="/settings" big>
					<Key />
					{#snippet active()}
						<Key weight="fill" />
					{/snippet}
				</TabLink>
			{/if}
			<TabLink
				title="Appearance"
				href="/settings/appearance"
				matcher={(page, href) => !!href && page.url.pathname.startsWith(href)}
				big
			>
				<PaintBrushBroad />
				{#snippet active()}
					<PaintBrushBroad weight="fill" />
				{/snippet}
			</TabLink>
			{#if $user}
				<TabLink
					title={$t("common.editors")}
					href={$pendingEditorFor ? "/settings/editors/editing-for" : "/settings/editors"}
					matcher={(page, _) => page.url.pathname.startsWith("/settings/editors")}
					big
				>
					<PencilSimple />
					{#snippet active()}
						<PencilSimple weight="fill" />
					{/snippet}
					{#snippet iconRight()}
						<NumberBadge count={$pendingEditorFor} />
					{/snippet}
				</TabLink>
				{#if $user.billing.subscriptionInfo.activePeriod?.providerId?.provider !== SubscriptionProvider.PayPal}
					<TabLink
						title={$t("pages.settings.billing.title")}
						href={PUBLIC_STRIPE_CUSTOMER_PORTAL}
						big
					>
						<CreditCard />
						{#snippet active()}
							<CreditCard weight="fill" />
						{/snippet}
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</TabLink>
				{/if}
			{/if}
		</nav>
		{#if $user}
			<div class="account hide-on-mobile">
				<UserProfilePicture user={$user} size={2.5 * 16} />
				<span class="name">
					<UserName user={$user} enablePaintDialog />
				</span>
			</div>
		{/if}
	</aside>
	<div class="content">
		<div class="width-wrapper">
			{@render children()}
		</div>
	</div>
</div>

<style lang="scss">
	.account {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;

		.name {
			font-weight: 600;
		}
	}

	// Only desktop
	@media screen and (min-width: 961px) {
		.content {
			overflow: auto;
			scrollbar-gutter: stable;
		}
	}

	.width-wrapper {
		margin-inline: auto;
		max-width: 80rem;

		display: flex;
		flex-direction: column;
		gap: 2rem;
	}
</style>
