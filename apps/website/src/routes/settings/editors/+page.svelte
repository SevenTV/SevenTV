<script lang="ts">
	import Editors from "$/components/settings/editors.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { graphql } from "$/gql";
	import type { UserEditor } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { user } from "$/lib/auth";
	import TabLink from "$/components/tab-link.svelte";
	import NumberBadge from "$/components/number-badge.svelte";
	import { pendingEditorFor } from "$/lib/auth";
	import { t } from "svelte-i18n";

	async function queryEditors(userId: string) {
		const res = await gqlClient().query(
			graphql(`
				query UserEditors($userId: Id!) {
					users {
						user(id: $userId) {
							editors {
								userId
								editorId
								editor {
									id
									mainConnection {
										platformDisplayName
										platformAvatarUrl
									}
									style {
										activeProfilePicture {
											images {
												url
												mime
												size
												width
												height
												scale
												frameCount
											}
										}
										activePaint {
											id
											name
											data {
												layers {
													id
													ty {
														__typename
														... on PaintLayerTypeSingleColor {
															color {
																hex
															}
														}
														... on PaintLayerTypeLinearGradient {
															angle
															repeating
															stops {
																at
																color {
																	hex
																}
															}
														}
														... on PaintLayerTypeRadialGradient {
															repeating
															stops {
																at
																color {
																	hex
																}
															}
															shape
														}
														... on PaintLayerTypeImage {
															images {
																url
																mime
																size
																scale
																width
																height
																frameCount
															}
														}
													}
													opacity
												}
												shadows {
													color {
														hex
													}
													offsetX
													offsetY
													blur
												}
											}
										}
									}
									highestRoleColor {
										hex
									}
								}
								state
								permissions {
									superAdmin
									emoteSet {
										admin
										create
										manage
									}
									emote {
										admin
										create
										manage
										transfer
									}
									user {
										admin
										manageBilling
										manageEditors
										managePersonalEmoteSet
										manageProfile
									}
								}
								updatedAt
							}
						}
					}
				}
			`),
			{ userId },
			{ requestPolicy: "network-only" },
		);

		if (!res.data?.users.user) {
			throw res.error?.message;
		}

		return res.data.users.user.editors as UserEditor[];
	}

	let editors = $derived($user ? queryEditors($user.id) : Promise.race([]));
</script>

{#await editors}
	<div class="spinner-container">
		<Spinner />
	</div>
{:then editors}
	<Editors userId={$user!.id} {editors} tab="editors">
		<div class="link-list">
			<TabLink title={$t("common.editors")} href="/settings/editors" />
			<NumberBadge count={$pendingEditorFor}>
				<TabLink
					title={$t("pages.settings.editors.editing_for")}
					href="/settings/editors/editing-for"
				/>
			</NumberBadge>
		</div>
	</Editors>
{/await}

<style lang="scss">
	.spinner-container {
		margin-inline: auto;
	}

	.link-list {
		display: flex;
		background-color: var(--bg-light);
		border-radius: 0.5rem;
		padding: 0.3rem;
		gap: 0.3rem;
	}
</style>
