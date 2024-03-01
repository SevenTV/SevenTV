<script lang="ts">
	import Button from "$/components/button.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
	import { discoverFollowingLayout } from "$/lib/stores";
	import { DotsThree, Plus } from "phosphor-svelte";
</script>

<div class="uploads {$discoverFollowingLayout}">
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
		</div>
	</div>
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
		</div>
	</div>
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
		</div>
	</div>
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
		</div>
	</div>
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
			<EmotePreview bg="light" />
		</div>
	</div>
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
		</div>
	</div>
	<div class="upload">
		<Button style="position: absolute; top: 0.5rem; right: 0.5rem;">
			<DotsThree slot="icon" />
		</Button>
		<div class="info">
			<div class="profile-picture"></div>
			<span>
				<span class="username">forsen</span>
				<span class="time"> · 2 hours ago</span>
			</span>
			<span>
				<Plus color="var(--primary)" size="1rem" />
				<span class="details">Added new emotes to Emote Set</span>
			</span>
		</div>
		<div class="emotes">
			<EmotePreview bg="light" />
		</div>
	</div>
</div>

<style lang="scss">
	.uploads {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;

		&.big-grid {
			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(28rem, 1fr));
			gap: 1rem;
		}

		&.list {
			display: flex;
			flex-direction: column;
			gap: 1rem;

			.emotes {
				display: none;
			}

			.upload {
				padding: 0.5rem 1rem;
			}
		}
	}

	.upload {
		padding: 1.5rem 2rem;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		position: relative;
	}

	.info {
		display: grid;
		grid-template-columns: auto 1fr;
		align-items: center;
		column-gap: 1rem;

		font-size: 0.875rem;

		.profile-picture {
			grid-row: span 2;

			width: 3rem;
			height: 3rem;
			border-radius: 50%;
			background-color: var(--preview);
		}

		.username {
			font-weight: 600;
		}

		.time {
			color: var(--text-light);
		}

		.details {
			font-weight: 300;
		}
	}

	.emotes {
		margin-top: 1rem;

		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		justify-content: center;
		align-items: center;
	}
</style>
