<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { user } from "$/lib/auth";
	import { Key, PencilSimple, Bell, CreditCard, Prohibit, MagnifyingGlass } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import Spinner from "$/components/spinner.svelte";
	import UserName from "$/components/user-name.svelte";
	import UserProfilePicture from "$/components/user-profile-picture.svelte";

	let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
	<title>{$t("common.settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#if $user === undefined}
	<Spinner />
{:else if $user === null}
	<SignInDialog mode="shown-without-close" />
{:else}
	<div class="side-bar-layout">
		<aside class="side-bar">
			<h1>{$t("common.settings")}</h1>
			<nav class="link-list">
				<TabLink title={$t("pages.settings.account.title")} href="/settings" big>
					<Key />
					{#snippet active()}
						<Key weight="fill" />
					{/snippet}
				</TabLink>
				<TabLink
					title={$t("common.editors")}
					href="/settings/editors"
					matcher={(page, href) => !!href && page.url.pathname.startsWith(href)}
					big
				>
					<PencilSimple />
					{#snippet active()}
						<PencilSimple weight="fill" />
					{/snippet}
				</TabLink>
				<TabLink title={$t("pages.settings.billing.title")} href="/settings/billing" big>
					<CreditCard />
					{#snippet active()}
						<CreditCard weight="fill" />
					{/snippet}
				</TabLink>
			</nav>
			<div class="account hide-on-mobile">
				<UserProfilePicture user={$user} size={2.5 * 16} />
				<span class="name">
					<UserName user={$user} enablePaintDialog />
				</span>
			</div>
		</aside>
		<div class="content">
			<div class="width-wrapper">
				{@render children()}
			</div>
		</div>
	</div>
{/if}

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
