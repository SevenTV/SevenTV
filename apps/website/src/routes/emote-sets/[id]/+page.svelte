<script lang="ts">
	import Tags from "$/components/emotes/tags.svelte";
	import Flags, { emoteSetToFlags } from "$/components/flags.svelte";
	import { type DialogMode } from "$/components/dialogs/dialog.svelte";
	import { t } from "svelte-i18n";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import {
		EmoteSetKind,
		UserEditorState,
		type EmoteSet,
		type EmoteSetEmoteSearchResult,
		type User,
	} from "$/gql/graphql";
	import Button from "$/components/input/button.svelte";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import { emotesLayout } from "$/lib/layout";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import {
		CaretDown,
		Lightning,
		LightningSlash,
		MagnifyingGlass,
		NotePencil,
	} from "phosphor-svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { untrack } from "svelte";
	import { refreshUser, user } from "$/lib/auth";
	import { setActiveSet } from "$/lib/userMutations";
	import Spinner from "$/components/spinner.svelte";
	import EditEmoteSetDialog from "$/components/dialogs/edit-emote-set-dialog.svelte";
	import DropDown from "$/components/drop-down.svelte";
	import UserName from "$/components/user-name.svelte";
	import UserProfilePicture from "$/components/user-profile-picture.svelte";
	import HideOn from "$/components/hide-on.svelte";

	let { data }: { data: EmoteSet } = $props();

	// let selectionMode = $state(false);
	// let selectionMap = $state({});
	let editDialogMode: DialogMode = $state("hidden");
	// let copyEmotesDialogMode: DialogMode = $state("hidden");
	// let removeEmotesDialogMode: DialogMode = $state("hidden");

	let query = $state("");

	let timeout: NodeJS.Timeout | number | undefined; // not reactive

	async function queryEmotes(
		query: string | undefined,
		page: number,
		perPage: number,
	): Promise<EmoteSetEmoteSearchResult> {
		// Small timeout to prevent spamming requests when user is typing
		return new Promise((resolve, reject) => {
			if (timeout) {
				clearTimeout(timeout);
			}

			timeout = setTimeout(async () => {
				const res = await gqlClient()
					.query(
						graphql(`
							query EmotesInSet(
								$id: Id!
								$query: String
								$page: Int!
								$perPage: Int!
								$isDefaultSetSet: Boolean!
								$defaultSetId: Id!
							) {
								emoteSets {
									emoteSet(id: $id) {
										emotes(query: $query, page: $page, perPage: $perPage) {
											__typename
											items {
												alias
												flags {
													zeroWidth
												}
												emote {
													id
													defaultName
													owner {
														mainConnection {
															platformDisplayName
														}
														style {
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
													flags {
														# animated
														# approvedPersonal
														defaultZeroWidth
														# deniedPersonal
														# nsfw
														# private
														publicListed
													}
													images {
														url
														mime
														size
														scale
														width
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
												}
											}
											totalCount
											pageCount
										}
									}
								}
							}
						`),
						{
							id: data.id,
							query: query,
							page,
							perPage,
							isDefaultSetSet: !!$defaultEmoteSet,
							defaultSetId: $defaultEmoteSet ?? "",
						},
					)
					.toPromise();

				if (res.error || !res.data) {
					reject(res.error);
					return;
				}

				const emotes = res.data.emoteSets.emoteSet?.emotes;

				if (!emotes) {
					reject(new Error("No emotes found"));
					return;
				}

				resolve(emotes as EmoteSetEmoteSearchResult);
			}, 200);
		});
	}

	let loader: ReturnType<typeof EmoteLoader>;

	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		query; // trigger reactivity when query changes
		untrack(() => {
			loader?.reset();
		});
	});

	let setActiveLoading = $state(false);
	// svelte-ignore non_reactive_update
	let enableDropdown: ReturnType<typeof DropDown>;

	async function setAsActiveSet(userId: string, id?: string) {
		enableDropdown?.close();
		setActiveLoading = true;

		await setActiveSet(userId, id);

		refreshUser();

		setActiveLoading = false;
	}

	let editingFor = $derived.by(() => {
		const users: User[] = [];
		if ($user) {
			users.push($user);
		}
		const editorFor = $user?.editorFor;
		for (const user of editorFor ?? []) {
			if (user.user) {
				users.push(user.user);
			}
		}
		return users;
	});
</script>

<svelte:head>
	<title>{data.name} - {$t("page_titles.suffix")}</title>
</svelte:head>

<EditEmoteSetDialog bind:mode={editDialogMode} bind:data />
<!-- <CopyEmotesDialog bind:mode={copyEmotesDialogMode} />
<RemoveEmotesDialog bind:mode={removeEmotesDialogMode} /> -->
<div class="layout">
	<div class="set-info">
		{#if data.owner}
			<a href="/users/{data.owner.id}" class="user-info">
				<UserProfilePicture user={data.owner} size={2 * 16} />
				<HideOn mobile>
					<UserName user={data.owner} />
				</HideOn>
			</a>
		{/if}
		<h1>{data.name}</h1>
		<Flags flags={emoteSetToFlags(data, $user, $defaultEmoteSet)} style="justify-self: end;" />
		<Tags tags={data.tags} style="grid-column: 2;" />
		{#if data.capacity}
			<div class="progress">
				<progress value={data.emotes.totalCount} max={data.capacity}></progress>
				{data.emotes.totalCount}/{data.capacity}
			</div>
		{/if}
	</div>
	<div class="controls">
		<div class="buttons">
			<!-- <Button secondary onclick={() => (selectionMode = !selectionMode)} hideOnDesktop>
				{$t("labels.select")}
				{#snippet iconRight()}
					<Toggle bind:value={selectionMode} />
				{/snippet}
			</Button> -->
			{#if $user}
				<!-- <HideOn mobile={selectionMode}> -->
				{#if data.kind === EmoteSetKind.Normal && ($user.id === data.owner?.id || $user.permissions.user.manageAny || $user.editorFor.some((editor) => editor?.editorId === $user?.id && editor.permissions.user.manageProfile))}
					<DropDown align="left" bind:this={enableDropdown}>
						<Button primary disabled={setActiveLoading}>
							Enable
							{#snippet icon()}
								{#if setActiveLoading}
									<Spinner />
								{:else}
									<Lightning />
								{/if}
							{/snippet}
							{#snippet iconRight()}
								<CaretDown />
							{/snippet}
						</Button>
						{#snippet dropdown()}
							<div class="button-list">
								{#each editingFor as user}
									{@const isActive = user.style.activeEmoteSetId === data.id}
									<Button
										onclick={() => setAsActiveSet(user.id, isActive ? undefined : data.id)}
										disabled={setActiveLoading}
									>
										{#if isActive}
											{$t("labels.disable")} for <UserName {user} />
										{:else}
											{$t("labels.enable")} for <UserName {user} />
										{/if}
										{#snippet icon()}
											{#if isActive}
												<LightningSlash />
											{:else}
												<Lightning />
											{/if}
										{/snippet}
									</Button>
								{/each}
							</div>
						{/snippet}
					</DropDown>
				{/if}
				{#if $user.permissions.emoteSet.manage && ($user.id === data.owner?.id || $user.permissions.emoteSet.manageAny || data.owner?.editors.some((editor) => editor?.editorId === $user?.id && editor.state === UserEditorState.Accepted && editor.permissions.emoteSet.manage))}
					<Button secondary hideOnMobile onclick={() => (editDialogMode = "shown")}>
						{$t("labels.edit")}
						{#snippet iconRight()}
							<NotePencil />
						{/snippet}
					</Button>
					<Button secondary hideOnDesktop onclick={() => (editDialogMode = "shown")}>
						{#snippet iconRight()}
							<NotePencil />
						{/snippet}
					</Button>
				{/if}
				<!-- </HideOn> -->
			{/if}
			<!-- <Button secondary hideOnMobile>
				{$t("pages.emote_set.copy_set")}
				{#snippet iconRight()}
					<Copy />
				{/snippet}
			</Button>
			{#if !selectionMode}
				<Button secondary hideOnDesktop>
					{#snippet iconRight()}
						<Copy />
					{/snippet}
				</Button>
			{/if}
			<Button secondary onclick={() => (selectionMode = !selectionMode)} hideOnMobile>
				{$t("labels.selection_mode")}
				{#snippet iconRight()}
					<Toggle bind:value={selectionMode} />
				{/snippet}
			</Button>
			{#if selectionMode}
				<Button onclick={() => (copyEmotesDialogMode = "shown")}>
					{#snippet icon()}
						<Copy />
					{/snippet}
				</Button>
				<Button>
					{#snippet icon()}
						<NotePencil />
					{/snippet}
				</Button>
				<Button onclick={() => (removeEmotesDialogMode = "shown")}>
					{#snippet icon()}
						<Trash />
					{/snippet}
				</Button>
			{/if} -->
		</div>
		<div class="buttons">
			<!-- <Select
				options={[
					{ value: "none", label: $t("labels.no_filters") },
					{ value: "filters", label: $t("labels.filters") },
				]}
			/> -->
			<TextInput placeholder={$t("labels.search")} bind:value={query}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput>
			<LayoutButtons bind:value={$emotesLayout} />
		</div>
	</div>
	<div class="content">
		<!-- <EmoteLoader
			bind:this={loader}
			load={(page, perPage) => queryEmotes(query || undefined, page, perPage)}
			scrollable={false}
			{selectionMode}
			bind:selectionMap
		/> -->
		<EmoteLoader
			bind:this={loader}
			load={(page, perPage) => queryEmotes(query || undefined, page, perPage)}
			scrollable={false}
		/>
	</div>
</div>

<style lang="scss">
	.layout {
		padding: 1.25rem;
		padding-bottom: 0;
		height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.user-info {
		display: flex;
		align-items: center;
		column-gap: 0.5rem;

		text-decoration: none;
		font-weight: 500;
		color: var(--text);
	}

	.set-info {
		padding: 1rem;

		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: 0.75rem;
		align-items: center;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		h1 {
			grid-column: 2;
			text-align: center;
			font-size: 1.125rem;
			font-weight: 500;

			overflow: hidden;
			text-overflow: ellipsis;
			white-space: nowrap;
		}

		.progress {
			grid-column: 1 / -1;

			display: flex;
			align-items: center;
			gap: 0.75rem;

			font-size: 0.875rem;
			font-weight: 500;

			progress {
				flex-grow: 1;
			}
		}
	}

	.controls {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap-reverse;
		justify-content: space-between;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.button-list {
		display: flex;
		flex-direction: column;
	}

	.content {
		flex-grow: 1;

		overflow: auto;
		scrollbar-gutter: stable;
		margin-right: -1.25rem;
		padding-right: 1.25rem;
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
			padding-bottom: 0;
		}

		.content {
			margin-right: 0;
			padding-right: 0;
		}
	}
</style>
