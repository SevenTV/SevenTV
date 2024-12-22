<script lang="ts">
	import Editors from "$/components/settings/editors.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { user } from "$/lib/auth";
	import type { UserEditor } from "$/gql/graphql";
	import Spinner from "$/components/spinner.svelte";
	import NumberBadge from "$/components/number-badge.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { t } from "svelte-i18n";
	import { pendingEditorFor } from "$/lib/auth";

	async function queryEditors(userId: string) {
		const res = await gqlClient().query(
			graphql(`
				query UserEditorFor($userId: Id!) {
					users {
						user(id: $userId) {
							editorFor {
								userId
								editorId
								user {
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
									emoteSet {
										manage
									}
									emote {
										manage
									}
									user {
										manageProfile
										manageEditors
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

		return res.data.users.user.editorFor as UserEditor[];
	}

	let editors = $derived($user ? queryEditors($user.id) : Promise.race([]));
</script>

{#await editors}
	<div class="spinner-container">
		<Spinner />
	</div>
{:then editors}
	<Editors userId={$user!.id} {editors} tab="editing-for">
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
