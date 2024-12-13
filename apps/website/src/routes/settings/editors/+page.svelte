<script lang="ts">
	import Editors from "$/components/settings/editors.svelte";
	import Spinner from "$/components/spinner.svelte";
	import { graphql } from "$/gql";
	import type { UserEditor } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { user } from "$/lib/auth";

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

		return res.data.users.user.editors as UserEditor[];
	}

	let editors = $derived(queryEditors($user!.id));
</script>

{#await editors}
	<div class="spinner-container">
		<Spinner />
	</div>
{:then editors}
	<Editors {editors} tab="editors" />
{/await}

<style lang="scss">
	.spinner-container {
		margin-inline: auto;
	}
</style>
