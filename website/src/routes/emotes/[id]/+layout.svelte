<script lang="ts">
	import Button from "$/components/button.svelte";
	import Tag from "$/components/emotes/tag.svelte";
	import ImagePreview from "$/components/image-preview.svelte";
	import {
		ArrowBendDownRight,
		Plus,
		FolderPlus,
		NotePencil,
		CaretDown,
		GlobeSimple,
		StackSimple,
		Fire,
	} from "phosphor-svelte";
	import type { LayoutData } from "./$types";

	export let data: LayoutData;
</script>

<svelte:head>
	<title>{data.id} - 7TV</title>
</svelte:head>

<div class="layout">
	<div>
		<div class="top-bar">
			<a href="/user/ayyybubu" class="user-info">
				<img
					src="/test-profile-pic.jpeg"
					width="44"
					height="44"
					alt="profile"
					class="profile-picture"
				/>
				<span class="username">ayyybubu</span>
				<div class="artists">
					<ArrowBendDownRight size="0.75rem" color="var(--text-light)" />
					<a href="/user/ayyybubu" class="profile">
						<img
							src="/test-profile-pic.jpeg"
							width="16"
							height="16"
							alt="ayyybubu"
							title="ayyybubu"
							class="artist-picture"
						/>
					</a>
				</div>
			</a>
			<div class="flags">
				<span class="flag">
					<GlobeSimple size="1rem" />
					<span class="hide-on-mobile">Global</span>
				</span>
				<span class="flag">
					<Fire size="1rem" />
					<span class="hide-on-mobile">Trending</span>
				</span>
				<span class="flag">
					<StackSimple size="1rem" />
					<span class="hide-on-mobile">Overlay</span>
				</span>
			</div>
		</div>

		<div class="emote-info">
			<div class="heading">
				<h1>{data.id}</h1>
				<div class="tags">
					<Tag>lorem</Tag>
					<Tag>ipsum</Tag>
					<Tag>dolor</Tag>
				</div>
			</div>
			<div class="previews">
				<ImagePreview size={32} />
				<ImagePreview size={64} />
				<ImagePreview size={96} />
				<ImagePreview size={128} />
			</div>
			<div class="buttons">
				<Button primary>
					<Plus slot="icon" />
					Use Emote
				</Button>
				<Button secondary>
					<FolderPlus slot="icon" />
					Add to...
				</Button>
				<Button secondary hideOnMobile>
					<NotePencil slot="icon" />
					Edit
				</Button>
				<Button secondary hideOnDesktop>
					<NotePencil slot="icon" />
				</Button>
				<Button secondary hideOnMobile>
					More
					<CaretDown slot="icon-right" />
				</Button>
				<Button secondary hideOnDesktop>
					<CaretDown slot="icon" />
				</Button>
			</div>
		</div>
	</div>
	<div class="tabs">
		<slot />
	</div>
</div>

<style lang="scss">
	.layout {
		width: 100%;
		max-width: 80rem;
		margin-inline: auto;

		padding: 1.25rem;
		min-height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		& > * {
			background-color: var(--bg-medium);
			border: 1px solid var(--layout-border);
			border-radius: 0.5rem;
			padding: 1rem;
		}
	}

	.top-bar {
		align-self: stretch;

		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
	}

	.user-info {
		display: grid;
		grid-template-columns: auto auto;
		grid-template-rows: auto auto;
		align-items: center;
		column-gap: 0.5rem;
		row-gap: 0.25rem;
		text-decoration: none;

		.profile-picture {
			grid-row: 1 / -1;

			width: 2.75rem;
			height: 2.75rem;
			border-radius: 50%;
			border: 2px solid var(--staff);
		}

		.username {
			color: var(--staff);
			font-weight: 500;
		}

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

	.flags {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		.flag {
			display: flex;
			align-items: center;
			gap: 0.5rem;
			padding: 0.3rem 0.5rem 0.3rem 0.75rem;
			background-color: var(--secondary);
			border-radius: 0.5rem;

			color: var(--text-light);
			font-size: 0.75rem;
			font-weight: 500;
		}
	}

	.emote-info {
		margin-top: 0.5rem;

		.heading {
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: 1rem;

			h1 {
				font-size: 1.25rem;
				font-weight: 600;
			}

			.tags {
				display: flex;
				gap: 0.5rem;
			}
		}

		display: flex;
		flex-direction: column;
		justify-content: space-between;
		align-items: center;
		gap: 2rem;

		.previews {
			display: flex;
			align-items: flex-end;
			gap: 2.25rem;
		}

		.buttons {
			display: flex;
			gap: 0.5rem;
		}
	}

	.tabs {
		flex-grow: 1;
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
		}

		.emote-info .previews {
			gap: 0.75rem;
		}

		.flags .flag {
			padding: 0.3rem 0.5rem;
		}
	}
</style>
