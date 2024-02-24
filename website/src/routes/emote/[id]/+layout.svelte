<script lang="ts">
	import ImagePreview from "$/components/image-preview.svelte";
	import TabLink from "$/components/profile/tab-link.svelte";
	import Activity from "$/components/emotes/activity.svelte";
	import Tag from "$/components/emotes/tag.svelte";
	import type { LayoutData } from "./$types";
	import {
		ArrowBendDownRight,
		Lightning,
		Plus,
		FolderPlus,
		NotePencil,
		CaretDown,
		Users,
		ChartLineUp,
		Graph,
		ChatText,
	} from "phosphor-svelte";

	export let data: LayoutData;
</script>

<svelte:head>
	<title>{data.name} - 7TV</title>
</svelte:head>

<div class="content">
	<aside class="top-space">
		<div class="user-info hide-on-mobile">
			<a href="/user/{data.author}" class="profile" >
				<img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
			</a>
			<div class="artist-info">
				<a href="/user/{data.author}" class="profile" >
					<span class="user-name">
						{data.author}
					</span>
				</a>
				{#if data.artists.length}
					<div class="artists">
						<ArrowBendDownRight size="0.75rem" color="var(--secondary-light)"/>
						{#each data.artists as artist}
							<a href="/user/{artist.login}" class="profile" >
								<img src="{artist.avatar}" alt="{artist.displayName}" title="{artist.displayName}" class="artist-picture" />
							</a>
						{/each}
					</div>
				{/if}
			</div>
		</div>
		<div class="emote-info">
			<div class="title">
				{data.name}
			</div>
			<div class="tags">
				{#each data.tags as tag}
					<Tag name={tag} />
				{/each}
			</div>
			<div class="previews">
				<div class="32x32">
					<ImagePreview size={32}/>
					<span class="size-text">32x32</span>
				</div>
				<div class="64x64">
					<ImagePreview size={64}/>
					<span class="size-text">64x64</span>
				</div>
				<div class="96x96">
					<ImagePreview size={96}/>
					<span class="size-text">96x96</span>
				</div>
				<div class="128x128">
					<ImagePreview size={128}/>
					<span class="size-text">128x128</span>
				</div>
			</div>
			<div class="buttons">
				<button class="button icon-left primary grow">
					<Plus />
					Use Emote
				</button>
				<button class="button icon-left secondary grow">
					<FolderPlus />
					Add to...
				</button>
				<button class="button icon-left secondary grow">
					<NotePencil />
					Edit
				</button>
				<button class="button icon-rigth secondary grow">
					<span class="hide-on-mobile">
						More
					</span>
					<CaretDown />
				</button>
			</div>
		</div>
		<div class="right-side hide-on-mobile">
			<Lightning size="1rem"/>
			<span class="credits">16 credits</span>
		</div>
	</aside>
	<div class="bottom-space">
		<div class="tabs">
			<div class="buttons">
				<TabLink title="Channels ({data.channels})" href="/emote/{data.id}">
					<Users />
					<svelte:fragment slot="active">
						<Users weight="fill" />
					</svelte:fragment>
				</TabLink>
				<TabLink title="Statistics" href="/emote/{data.id}/statistics">
					<ChartLineUp />
					<svelte:fragment slot="active">
						<ChartLineUp weight="fill" />
					</svelte:fragment>
				</TabLink>
				<TabLink title="Suggested Emotes" href="/emote/{data.id}/suggested-emotes">
					<Graph />
					<svelte:fragment slot="active">
						<Graph weight="fill" />
					</svelte:fragment>
				</TabLink>
				<TabLink title="Mod Comments" href="/emote/{data.id}/mod-comments">
					<ChatText />
					<svelte:fragment slot="active">
						<ChatText weight="fill" />
					</svelte:fragment>
				</TabLink>
			</div>
			<div class="tab-content">
				<slot />
			</div>
		</div>
		<div class="activity hide-on-mobile">
			<span class="title">
				Activity
			</span>
			<div class="activity-events">
				{#each data.activity as event, index}
					<Activity event={{ ...event, oldName: event.oldName || "oldName" }} />
					{#if index !== data.activity.length - 1}
						<hr />
					{/if}
				{/each}
			</div>
		</div>
	</div>
</div>

<style lang="scss">

	.title {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 1rem;
	}

	a:hover {
		text-decoration: none;
	}

	.user-name {
		font-weight: 700;
		color: var(--text);
	}

	.content {
		padding: 1rem 2rem 0 2rem;

		.top-space {
			display: flex;
			background-color: var(--bg-medium);
			border-radius: 0.5rem;
			padding: 2rem;

			.user-info {
				display: flex;
				gap: 0.625rem;
				width: 30%;

				.profile-picture {
					width: 2.75rem;
					height: 2.75rem;
					border-radius: 50%;
					border: 2px solid var(--staff);
				}

				.user-name {
					color: var(--staff);
				}

				.artist-info {
					display: flex;
					flex-direction: column;
					gap: 0.25rem;

					.artists {
						display: flex;
						gap: 0.25rem;

						.artist-picture {
							width: 1rem;
							height: 1rem;
							border-radius: 50%;
							border: 1px solid var(--text);
						}
					}

				}
			}

			.emote-info {
				display: grid;
				justify-items: center;
				width: 40%;

				.title {
					align-self: center;
					color: var(--text);
				}

				.tags {
					align-self: center;
					display: flex;
					gap: 0.5rem;
				}

				.previews {
					display: flex;
					gap: 2rem;
					align-items: flex-end;
					margin: 2rem 0;

					.size-text {
						font-weight: 400;
						margin-top: 1rem;
						display: flex;
						justify-content: center;
						font-size: 0.6rem;
						color: var(--text-lighter);
					}
				}

				.buttons {
					display: flex;
					gap: 1rem;
				}
			}

			.right-side {
				width: 30%;
				display: flex;
				justify-content: flex-end;

				.credits {
					font-size: 1rem;
					font-weight: 500;
					color: var(--text);
				}
			}
		}

		.bottom-space {
			gap: 1rem;
			display: flex;
			margin-top: 1rem;

			.tabs {
				background-color: var(--bg-medium);
				border-radius: 0.5rem;
				margin-bottom: 1rem;
				min-height: 30rem;
				padding: 2rem;
				width: 70%;

				.buttons {
					justify-content: center;
					display: flex;
					gap: 2rem;
					flex-wrap: wrap;
					gap: 0.5rem;
					user-select: none;

					-ms-overflow-style: none;
					scrollbar-width: none;
					&::-webkit-scrollbar {
						display: none;
					}
				}
			}

			.activity {
				background-color: var(--bg-medium);
				border-radius: 0.5rem;
				margin-bottom: 1rem;
				padding: 2rem;
				width: 30%;
			}
		}
	}

	@media screen and (max-width: 960px) {

		.content {
			padding: 1rem 1.5rem 0 1.5rem;

			.top-space {
				padding: 1.5rem;
				flex-direction: column;

				.emote-info {
					width: 100%;
					justify-content: center;
					gap: 1rem;

					.tags {

						.button {
							padding: 0.5rem;
						}
					}

					.previews {
						gap: 0.5rem;
					}

					.buttons {
						gap: 0.5rem;

						.button {
							padding: 0.5rem;
						}
					}
				}
			}

			.bottom-space {
				.tabs {
					width: 100%;

					.buttons {
						justify-content: normal;
						gap: 0;
						margin-right: -1rem;
						padding-right: 1rem;
						overflow-x: auto;
						flex-wrap: nowrap;
					}
				}
			}
		}

	}
</style>
