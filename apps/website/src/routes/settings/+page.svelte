<script lang="ts">
	import { t } from "svelte-i18n";
	import { user } from "$/lib/auth";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import MainConnectionSelect from "$/components/settings/main-connection-select.svelte";
	import { type User } from "$/gql/graphql";
	import ProfilePictureSetting from "$/components/settings/profile-picture-setting.svelte";
	import ConnectionsSetting from "$/components/settings/connections-setting.svelte";
	import DeleteSessions from "$/components/settings/delete-sessions.svelte";
	import Spinner from "$/components/spinner.svelte";

	async function queryConnections(userId: string) {
		const res = await gqlClient().query(
			graphql(`
				query UserConnections($userId: Id!) {
					users {
						user(id: $userId) {
							mainConnection {
								platform
								platformId
							}
							connections {
								platform
								platformId
								platformDisplayName
							}
						}
					}
				}
			`),
			{ userId },
		);

		return res.data?.users.user as User;
	}

	let connections = $state<Promise<User> | undefined>($user ? queryConnections($user.id) : undefined);
</script>

<svelte:head>
	<title>{$t("page_titles.account_settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<section>
	<div>
		<h2>{$t("common.profile")}</h2>
		<span class="details">
			{$t("pages.settings.account.profile.details")}
		</span>
	</div>
	<div class="content">
		<span>
			<h3>{$t("pages.settings.account.profile.display_name")}</h3>
			<span class="details">{$t("pages.settings.account.profile.display_name_description")}</span>
		</span>
		{#if connections}
			<MainConnectionSelect bind:userData={connections} />
		{:else}
			<Spinner />
		{/if}
		<hr />
		<span>
			<h3>{$t("common.profile_picture")}</h3>
			<span class="details">{$t("pages.settings.account.profile.profile_picture_description")}</span
			>
		</span>
		<ProfilePictureSetting />
	</div>
</section>

<section>
	<div>
		<h2>{$t("common.connections")}</h2>
		<span class="details">
			Manage and link external accounts to streamline access and enhance interoperability within the
			platform
		</span>
	</div>
	<ul class="content connections">
		{#if connections}
			<ConnectionsSetting bind:userData={connections} />
		{:else}
			<Spinner />
		{/if}
	</ul>
</section>

<section>
	<div>
		<h2>{$t("pages.settings.account.security.title")}</h2>
		<span class="details">{$t("pages.settings.account.security.details")}</span>
	</div>
	<div class="content">
		<span>
			<h3>{$t("pages.settings.account.security.sign_out_everywhere")}</h3>
			<span class="details">
				{$t("pages.settings.account.security.sign_out_everywhere_details")}
			</span>
		</span>
		<DeleteSessions />
	</div>
</section>

<style lang="scss">
	@use "../../styles/settings.scss";

	h3 {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.connections {
		gap: 0.5rem;
	}
</style>
