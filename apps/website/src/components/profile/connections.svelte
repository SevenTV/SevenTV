<script lang="ts">
	import { Platform, type User, type UserConnection } from "$/gql/graphql";
	import { Link } from "phosphor-svelte";
	import DiscordLogo from "../icons/discord-logo.svelte";
	import KickLogo from "../icons/kick-logo.svelte";
	import TwitchLogo from "../icons/twitch-logo.svelte";
	import YoutubeLogo from "../icons/youtube-logo.svelte";
	import Button from "../input/button.svelte";

	export let user: User;
	export let big: boolean = false;

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
			{#if connection.platform === Platform.Twitch}
				<TwitchLogo slot="icon" />
			{:else if connection.platform === Platform.Google}
				<YoutubeLogo slot="icon" />
			{:else if connection.platform === Platform.Discord}
				<DiscordLogo slot="icon" />
			{:else if connection.platform === Platform.Kick}
				<KickLogo slot="icon" />
			{:else}
				<Link slot="icon" />
			{/if}
			<span>{connection.platformDisplayName}</span>
		</Button>
	{/if}
{/each}
