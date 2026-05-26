<script lang="ts">
	import {
		ArrowDown,
		ArrowsLeftRight,
		Folder,
		MagnifyingGlass,
		PaintBrush,
		SealCheck,
		Smiley,
		X,
		ArrowSquareOut,
	} from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import UserSearch from "../user-search.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import WarningDanger from "./warning-danger.svelte";

	import type { User } from "$/gql/graphql";
	import UserProfilePicture from "../user-profile-picture.svelte";
	import UserName from "../user-name.svelte";

	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { ProxyState } from "$/lib/proxy.svelte";

	import Role from "../users/role.svelte";
	import { filterRoles } from "$/lib/utils";
	import { onMount } from "svelte";
	import { t } from "svelte-i18n";
	import { goto } from "$app/navigation";
	import TextInput from "../input/text-input.svelte";
	import { user } from "$/lib/auth";

	let { mode = $bindable("hidden"), mainUser }: { mode: DialogMode; mainUser: User } = $props();

	let users = $state<Record<string, Promise<User> | undefined>>({
		main: undefined,
		second: undefined,
	});

	let searchQuery = $state("");
	let idQuery = $state("");

	let showConfirm = $state<DialogMode>("hidden");

	const onUserResults = (e: MouseEvent | null, user: User | string) => {
		searchQuery = "";
		idQuery = "";
		const userData = GetUser(typeof user == "string" ? user : user.id);
		if (userData.streamed.userRequest.value) users.second = userData.streamed.userRequest.value;
	};

	onMount(() => {
		const userData = GetUser(mainUser.id);
		if (userData.streamed.userRequest.value) users.main = userData.streamed.userRequest.value;
	});

	let mergeButtonDisabled = $state(true);

	$effect(() => {
		if (!users.main || !users.second) {
			mergeButtonDisabled = true;
			return;
		}
		Promise.all([users.main, users.second]).then(([main, second]) => {
			mergeButtonDisabled = main.id === second.id;
		});
	});

	let idQyeryButtonDisabled = $derived(idQuery.length < 26);

	function GetUser(id: string) {
		const req = gqlClient()
			.query(
				graphql(`
					query OneUserMerge($id: Id!) {
						users {
							user(id: $id) {
								id
								connections {
									platform
									platformId
									platformUsername
									platformDisplayName
								}
								mainConnection {
									platformDisplayName
									platformAvatarUrl
								}
								ownedEmotes {
									id
								}
								ownedEmoteSets {
									id
								}
								inventory(includeInaccessible: true) {
									badges {
										to {
											badge {
												id
											}
										}
									}
									paints {
										to {
											paint {
												id
											}
										}
									}
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
									activePaintId
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
									activeBadgeId
									activeBadge {
										id
										name
										description
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
									activeEmoteSet {
										id
										name
									}
								}
								highestRoleColor {
									hex
								}
								roles {
									name
									color {
										hex
									}
								}
							}
						}
					}
				`),
				{
					id,
				},
				{
					fetch,
				},
			)
			.toPromise()
			.then((res) => {
				if (res.error || !res.data) throw "Failed to load user";

				if (!res.data.users.user) throw "User not found";

				return res.data.users.user as User;
			});

		const state = new ProxyState(req);

		return {
			id: id,
			streamed: {
				userRequest: state,
			},
		};
	}

	async function onConfirm() {
		Promise.all([users.main, users.second]).then(([main, second]) => {
			if (main && second) {
				const mergeReq = MergeUser(main.id, second.id);

				mergeReq.promise.then(() => goto(`/users/${second.id}`));
			}
		});
	}

	function MergeUser(srcId: string, targetId: string) {
		const req = gqlClient()
			.mutation(
				// note for any 7tv dev, please add account merge to v4
				graphql(`
					mutation MergeAccount($src: Id!, $target: Id!) {
						user(id: $src) {
							merge(id: $target)
						}
					}
				`),
				{
					src: srcId,
					target: targetId,
				},
				{
					url: "https://7tv.io/v3/gql",
				},
			)
			.toPromise()
			.then((res) => {
				if (res.error || !res.data) {
					console.error("GQL Error:", res.error);
					throw "Failed to merge user";
				}
				return res.data.user.merge;
			});

		const state = new ProxyState(req);

		return {
			id: srcId,
			promise: req,
			streamed: {
				userRequest: state,
			},
		};
	}
</script>

<WarningDanger bind:mode={showConfirm} confirm={onConfirm} />

{#snippet userDisplay(data: User)}
	<section id="user">
		<UserProfilePicture user={data} size={4.75 * 16}></UserProfilePicture>
		<aside id="info">
			<span class="name" style:color={data.highestRoleColor?.hex}>
				<UserName user={data} enablePaintDialog /> <Button style="padding: 0.15rem;" title="Open In New Tab" href="/users/{data.id}" target="_blank"><ArrowSquareOut /></Button>
			</span>
			<small>ID: {data.id}</small>
			<div class="roles">
				{#each filterRoles(data.roles) as role}
					<Role roleData={role} />
				{/each}
			</div>

			<section id="stats">
				<span
					><SealCheck size="0.75rem" />
					Badges: {data.inventory.badges.length.toLocaleString()}</span
				>
				<span
					><PaintBrush size="0.75rem" />
					Paints: {data.inventory.paints.length.toLocaleString()}</span
				>
				<span
					><Smiley size="0.75rem" />
					{$t("dialogs.editor.emotes")}: {data.ownedEmotes.length.toLocaleString()}</span
				>
				<span
					><Folder size="0.75rem" />
					{$t("common.emote_sets", { values: { count: 2 } })}: {data.ownedEmoteSets.length.toLocaleString()}</span
				>
			</section>
		</aside>
	</section>
{/snippet}

<Dialog width={35} bind:mode>
	<div class="layout">
		<h1>Account Merge</h1>
		<hr />
		{#await users.main then data}
			{#if data}
				{@render userDisplay(data)}
			{:else}
				<div class="name placeholder loading-animation">{$t("dialogs.editor.user")}</div>
			{/if}
		{/await}
		<span class="middle"><ArrowDown /> Merged Into <ArrowDown /></span>
		{#await users.second then data}
			{#if data}
				{@render userDisplay(data)}
			{:else}
				<div class="name placeholder loading-animation">{$t("dialogs.editor.user")}</div>
			{/if}
		{/await}
		<hr />
		<p>{$t("labels.search_users", { values: { count: 1 } })}</p>
		<UserSearch onresultclick={onUserResults} resulthref={() => ""} bind:query={searchQuery} />
		<p>{$t("pages.admin.users.id.subscription.period.id")}</p>
		<span>
			<TextInput maxlength={26} stretch bind:value={idQuery} />
			<Button primary disabled={idQyeryButtonDisabled} onclick={() => onUserResults(null, idQuery)}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
				<span>{$t("labels.proceed")}</span>
			</Button>
		</span>

		<section id="buttons">
			<Button secondary onclick={() => (mode = "hidden")}>
				{#snippet icon()}
					<X />
				{/snippet}
				<span>{$t("labels.cancel")}</span>
			</Button>
			<Button danger disabled={mergeButtonDisabled} onclick={() => (showConfirm = "shown")}>
				{#snippet icon()}
					<ArrowsLeftRight />
				{/snippet}
				<span>{$t("pages.emote.merge")}</span>
			</Button>
		</section>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		overflow-x: hidden;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	span {
		display: inline-flex;
		align-items: center;
		gap: 4px;

		&.middle {
			justify-content: center;
		}
	}

	#buttons {
		display: flex;
		justify-content: space-between;
		gap: 0.5rem;
	}

	#user {
		display: flex;
		align-items: center;

		gap: 1rem;

		#info {
			display: flex;
			justify-content: center;
			flex-direction: column;

			gap: 0.25rem;

			.roles {
				display: inline-flex;
				flex-wrap: wrap;

				gap: 0.5rem;
			}

			#stats {
				font-size: 0.75rem;
			}
		}
	}
</style>
