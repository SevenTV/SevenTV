<script lang="ts">
	import { Platform, type User, type UserConnection } from "$/gql/graphql";
	import { Link } from "phosphor-svelte";
	import DiscordLogo from "../icons/discord-logo.svelte";
	import KickLogo from "../icons/kick-logo.svelte";
	import TwitchLogo from "../icons/twitch-logo.svelte";
	import YoutubeLogo from "../icons/youtube-logo.svelte";
	import Button from "../input/button.svelte";

	let { user, big = false }: { user: User; big?: boolean } = $props();

	function connectionLink(connection: UserConnection): string | undefined {
		switch (connection.platform) {
			case Platform.Twitch:
				return `https://twitch.tv/${connection.platformUsername}`;
			case Platform.Google:
				return `https://youtube.com/@${connection.platformUsername}`;
			case Platform.Kick:
				return `https://kick.com/${connection.platformUsername}`;
			default:
				return undefined;
		}
	}
</script>

{#each user.connections as connection}
	{#if connection.platformDisplayName.length !== 0}
		<Button href={connectionLink(connection)} target="_blank" {big}>
			{#snippet icon()}
				{#if connection.platform === Platform.Twitch}
					<TwitchLogo />
				{:else if connection.platform === Platform.Google}
					<YoutubeLogo />
				{:else if connection.platform === Platform.Discord}
					<DiscordLogo />
				{:else if connection.platform === Platform.Kick}
					<KickLogo />
				{:else}
					<Link />
				{/if}
			{/snippet}
			<span class="name">
				{#if connection.platformDisplayName === connection.platformUsername}
					{connection.platformDisplayName}
				{:else}
					{connection.platformDisplayName} ({connection.platformUsername})
				{/if}
			</span>
		</Button>
	{/if}
{/each}

<style lang="scss">
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
