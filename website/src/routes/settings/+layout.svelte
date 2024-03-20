<script lang="ts">
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { signInDialogMode, user } from "$/lib/stores";
	import { Key, PencilSimple, Bell, CreditCard, Prohibit, MagnifyingGlass } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	$: if (!$user && !$signInDialogMode) {
		$signInDialogMode = DialogMode.ShownWithoutClose;
	}
</script>

<svelte:head>
	<title>{$t("common.settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#if $user}
	<div class="side-bar-layout">
		<aside class="side-bar">
			<h1>{$t("common.settings")}</h1>
			<TextInput placeholder={$t("labels.search")}>
				<MagnifyingGlass slot="icon" />
			</TextInput>
			<nav class="link-list">
				<TabLink title={$t("pages.settings.account.title")} href="/settings" big>
					<Key />
					<Key weight="fill" slot="active" />
				</TabLink>
				<TabLink
					title={$t("common.editors")}
					href="/settings/editors"
					matcher={(_id, url, href) => !!href && url.pathname.startsWith(href)}
					big
				>
					<PencilSimple />
					<PencilSimple weight="fill" slot="active" />
				</TabLink>
				<TabLink title={$t("common.notifications")} href="/settings/notifications" big>
					<Bell />
					<Bell weight="fill" slot="active" />
				</TabLink>
				<TabLink title={$t("pages.settings.blocked.title")} href="/settings/blocked" big>
					<Prohibit />
					<Prohibit weight="fill" slot="active" />
				</TabLink>
				<TabLink title={$t("pages.settings.billing.title")} href="/settings/billing" big>
					<CreditCard />
					<CreditCard weight="fill" slot="active" />
				</TabLink>
			</nav>
			<div class="account hide-on-mobile">
				<img
					class="profile-picture"
					src="/test-profile-pic.jpeg"
					alt="profile"
					width={2.5 * 16}
					height={2.5 * 16}
				/>
				<span class="name">ayyybubu</span>
			</div>
		</aside>
		<div class="content">
			<div class="width-wrapper">
				<slot />
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

		.profile-picture {
			border-radius: 50%;
			border: 2px solid var(--staff);
		}

		.name {
			color: var(--staff);
			font-weight: 600;
		}
	}

	// Only desktop
	@media screen and (min-width: 961px) {
		.content {
			overflow: auto;
			overflow: overlay;
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
