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
	import Button from "$/components/button.svelte";

	export let data: LayoutData;
</script>

<svelte:head>
	<title>{data.name} - 7TV</title>
</svelte:head>

<div class="content">
	<aside class="top-space">
		<div class="user-info hide-on-mobile">
			<a href="/user/{data.author}" class="profile">
				<img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
			</a>
			<div class="artist-info">
				<a href="/user/{data.author}" class="profile">
					<span class="user-name">
						{data.author}
					</span>
				</a>
				{#if data.artists.length}
					<div class="artists">
						<ArrowBendDownRight size="0.75rem" color="var(--secondary-light)" />
						{#each data.artists as artist}
							<a href="/user/{artist.login}" class="profile">
								<img
									src={artist.avatar}
									alt={artist.displayName}
									title={artist.displayName}
									class="artist-picture"
								/>
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
				<ImagePreview size={32} />
				<ImagePreview size={64} />
				<ImagePreview size={96} />
				<ImagePreview size={128} />
			</div>
			<div class="buttons">
				<Button secondary>
					<Plus slot="icon" />
					<span>
						Use
						<span class="hide-on-mobile">Emote</span>
					</span>
				</Button>
				<Button primary>
					<FolderPlus slot="icon" />
					Add to...
				</Button>
				<Button primary>
					<NotePencil slot="icon" />
					Edit
				</Button>
				<Button primary hideOnMobile>
					More
					<CaretDown slot="icon-right" />
				</Button>
				<Button primary hideOnDesktop>
					<CaretDown slot="icon" />
				</Button>
			</div>
		</div>
		<span class="credits hide-on-mobile">
			<Lightning size="1rem" />
			16 credits
		</span>
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
			<slot />
		</div>
		<div class="activity hide-on-mobile">
			<span class="title"> Activity </span>
			<div class="activity-events">
				<Activity activities={data.activity} />
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
			justify-content: space-between;
			align-items: flex-start;

			background-color: var(--bg-medium);
			border-radius: 0.5rem;
			padding: 2rem;

			.user-info {
				display: flex;
				gap: 0.62rem;

				.profile-picture {
					width: 2.75rem;
					height: 2.75rem;
					border-radius: 50%;
					border: 2px solid var(--staff);
				}

				.user-name {
					color: var(--staff);
					font-weight: 500;
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
				display: flex;
				flex-direction: column;
				align-items: center;
				gap: 1rem;

				.title {
					color: var(--text);
					font-weight: 600;
				}

				.tags {
					display: flex;
					gap: 0.5rem;
					flex-wrap: wrap;
					justify-content: center;
				}

				.previews {
					margin-block: 0.5rem;

					display: flex;
					gap: 2rem;
					align-items: flex-end;
				}

				.buttons {
					display: flex;
					gap: 1rem;
				}
			}

			.credits {
				display: flex;
				align-items: center;
				gap: 0.5rem;

				font-weight: 500;
			}
		}

		.bottom-space {
			display: flex;
			gap: 1rem;
			margin-top: 1rem;

			.tabs {
				flex-grow: 1;

				background-color: var(--bg-medium);
				border-radius: 0.5rem;
				margin-bottom: 1rem;
				min-height: 30rem;
				padding: 2rem;

				.buttons {
					display: flex;
					gap: 0.5rem;
					overflow-x: auto;
					margin-inline: -2rem;
					padding-left: 2rem;

					-ms-overflow-style: none;
					scrollbar-width: none;
					&::-webkit-scrollbar {
						display: none;
					}
				}
			}

			.activity {
				flex-basis: 30%;

				background-color: var(--bg-medium);
				border-radius: 0.5rem;
				margin-bottom: 1rem;
				padding: 2rem;
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

					.previews {
						gap: 0.5rem;
					}

					.buttons {
						gap: 0.5rem;
					}
				}
			}

			.bottom-space {
				.tabs {
					width: 100%;

					.buttons {
						gap: 0;
					}
				}
			}
		}
	}
</style>
