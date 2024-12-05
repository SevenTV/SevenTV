import type { Emote, EmoteFlagsInput } from "$/gql/graphql";
import { get } from "svelte/store";
import { defaultEmoteSet } from "./defaultEmoteSet";
import { graphql } from "$/gql";
import { gqlClient } from "./gql";
import { PUBLIC_REST_API_V4 } from "$env/static/public";
import { currentError, errorDialogMode } from "./error";
import { sessionToken } from "./auth";

export async function upload(
	data: Blob,
	name: string,
	tags: string[],
	zeroWidth: boolean,
	privateFlag: boolean,
) {
	const token = get(sessionToken);

	if (!token) {
		return undefined;
	}

	const metadata = {
		name,
		tags,
		default_zero_width: zeroWidth,
		private: privateFlag,
	};

	const formData = new FormData();

	formData.append("metadata", JSON.stringify(metadata));

	formData.append("file", data);

	const response = await fetch(`${PUBLIC_REST_API_V4}/emotes`, {
		method: "POST",
		body: formData,
		credentials: "include",
		headers: {
			Authorization: `Bearer ${token}`,
		},
	});

	const json = await response.json();

	if (response.ok) {
		return json;
	} else {
		currentError.set(`Failed to upload emote: ${json.error}`);
		errorDialogMode.set("shown");
	}
}

export async function updateName(id: string, name: string) {
	const defaultSet = get(defaultEmoteSet);

	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation UpdateEmoteName(
					$id: Id!
					$name: String!
					$isDefaultSetSet: Boolean!
					$defaultSetId: Id!
				) {
					emotes {
						emote(id: $id) {
							name(name: $name) {
								id
								defaultName
								owner {
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
									editors {
										editorId
										permissions {
											emote {
												manage
											}
										}
									}
								}
								tags
								flags {
									animated
									defaultZeroWidth
									publicListed
									approvedPersonal
									deniedPersonal
								}
								attribution {
									user {
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
										}
										highestRoleColor {
											hex
										}
									}
								}
								images {
									url
									mime
									size
									width
									height
									scale
									frameCount
								}
								ranking(ranking: TRENDING_WEEKLY)
								inEmoteSets(emoteSetIds: [$defaultSetId]) @include(if: $isDefaultSetSet) {
									emoteSetId
									emote {
										id
										alias
									}
								}
								deleted
							}
						}
					}
				}
			`),
			{
				id,
				name,
				isDefaultSetSet: !!defaultSet,
				defaultSetId: defaultSet ?? "",
			},
		)
		.toPromise();

	if (res.data?.emotes.emote) {
		return res.data.emotes.emote.name as Emote;
	}

	return undefined;
}

export async function updateTags(id: string, tags: string[]) {
	const defaultSet = get(defaultEmoteSet);

	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation UpdateEmoteTags(
					$id: Id!
					$tags: [String!]!
					$isDefaultSetSet: Boolean!
					$defaultSetId: Id!
				) {
					emotes {
						emote(id: $id) {
							tags(tags: $tags) {
								id
								defaultName
								owner {
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
									editors {
										editorId
										permissions {
											emote {
												manage
											}
										}
									}
								}
								tags
								flags {
									animated
									defaultZeroWidth
									publicListed
									approvedPersonal
									deniedPersonal
								}
								attribution {
									user {
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
										}
										highestRoleColor {
											hex
										}
									}
								}
								images {
									url
									mime
									size
									width
									height
									scale
									frameCount
								}
								ranking(ranking: TRENDING_WEEKLY)
								inEmoteSets(emoteSetIds: [$defaultSetId]) @include(if: $isDefaultSetSet) {
									emoteSetId
									emote {
										id
										alias
									}
								}
								deleted
							}
						}
					}
				}
			`),
			{
				id,
				tags,
				isDefaultSetSet: !!defaultSet,
				defaultSetId: defaultSet ?? "",
			},
		)
		.toPromise();

	if (res.data?.emotes.emote) {
		return res.data.emotes.emote.tags as Emote;
	}

	return undefined;
}

export async function updateFlags(id: string, flags: EmoteFlagsInput) {
	const defaultSet = get(defaultEmoteSet);

	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation UpdateEmoteFlags(
					$id: Id!
					$flags: EmoteFlagsInput!
					$isDefaultSetSet: Boolean!
					$defaultSetId: Id!
				) {
					emotes {
						emote(id: $id) {
							flags(flags: $flags) {
								id
								defaultName
								owner {
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
									editors {
										editorId
										permissions {
											emote {
												manage
											}
										}
									}
								}
								tags
								flags {
									animated
									defaultZeroWidth
									publicListed
									approvedPersonal
									deniedPersonal
								}
								attribution {
									user {
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
										}
										highestRoleColor {
											hex
										}
									}
								}
								images {
									url
									mime
									size
									width
									height
									scale
									frameCount
								}
								ranking(ranking: TRENDING_WEEKLY)
								inEmoteSets(emoteSetIds: [$defaultSetId]) @include(if: $isDefaultSetSet) {
									emoteSetId
									emote {
										id
										alias
									}
								}
								deleted
							}
						}
					}
				}
			`),
			{
				id,
				flags,
				isDefaultSetSet: !!defaultSet,
				defaultSetId: defaultSet ?? "",
			},
		)
		.toPromise();

	if (res.data?.emotes.emote) {
		return res.data.emotes.emote.flags as Emote;
	}

	return undefined;
}

export async function deleteEmote(id: string) {
	const defaultSet = get(defaultEmoteSet);

	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation DeleteEmote($id: Id!, $isDefaultSetSet: Boolean!, $defaultSetId: Id!) {
					emotes {
						emote(id: $id) {
							delete {
								id
								defaultName
								owner {
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
									editors {
										editorId
										permissions {
											emote {
												manage
											}
										}
									}
								}
								tags
								flags {
									animated
									defaultZeroWidth
									publicListed
									approvedPersonal
									deniedPersonal
								}
								attribution {
									user {
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
										}
										highestRoleColor {
											hex
										}
									}
								}
								images {
									url
									mime
									size
									width
									height
									scale
									frameCount
								}
								ranking(ranking: TRENDING_WEEKLY)
								inEmoteSets(emoteSetIds: [$defaultSetId]) @include(if: $isDefaultSetSet) {
									emoteSetId
									emote {
										id
										alias
									}
								}
								deleted
							}
						}
					}
				}
			`),
			{
				id,
				isDefaultSetSet: !!defaultSet,
				defaultSetId: defaultSet ?? "",
			},
		)
		.toPromise();

	if (res.data?.emotes.emote) {
		return res.data.emotes.emote.delete as Emote;
	}

	return undefined;
}
