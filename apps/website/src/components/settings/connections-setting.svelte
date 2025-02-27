<script lang="ts">
	import { Platform, type User, type UserConnection } from "$/gql/graphql";
	import { removeConnection } from "$/lib/userMutations";
	import { platformToValue } from "$/lib/utils";
	import { PUBLIC_REST_API_V4 } from "$env/static/public";
	import DiscordLogo from "../icons/discord-logo.svelte";
	import KickLogo from "../icons/kick-logo.svelte";
	import TwitchLogo from "../icons/twitch-logo.svelte";
	import YoutubeLogo from "../icons/youtube-logo.svelte";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import { t } from "svelte-i18n";
	import { refreshUser, user } from "$/lib/auth";

	let { userData = $bindable() }: { userData: Promise<User> } = $props();

	const PLATFORMS = [
		{
			id: Platform.Twitch,
			linkUrl: linkUrl("twitch"),
			name: "Twitch",
			icon: twitchLogo,
			linkingEnabled: true,
		},
		{
			id: Platform.Discord,
			linkUrl: linkUrl("discord"),
			name: "Discord",
			icon: discordLogo,
			linkingEnabled: true,
		},
		{
			id: Platform.Kick,
			linkUrl: linkUrl("kick"),
			name: "Kick",
			icon: kickLogo,
			linkingEnabled: false,
		},
		{
			id: Platform.Google,
			linkUrl: linkUrl("google"),
			name: "YouTube",
			icon: youtubeLogo,
			linkingEnabled: false,
		},
	];

	function linkUrl(platform: string) {
		return `${PUBLIC_REST_API_V4}/auth/link?platform=${platform}&return_to=/settings`;
	}

	let removeLoading = $state();

	async function removeUserConnection(platform: Platform, platformId: string) {
		removeLoading = platformToValue(platform, platformId);

		userData = removeConnection($user!.id, platform, platformId).then((data) =>
			data ? data : userData,
		);

		await userData;
		removeLoading = undefined;

		refreshUser();
	}
</script>

{#snippet discordLogo()}
	<DiscordLogo />
{/snippet}
{#snippet twitchLogo()}
	<TwitchLogo />
{/snippet}
{#snippet youtubeLogo()}
	<YoutubeLogo />
{/snippet}
{#snippet kickLogo()}
	<KickLogo />
{/snippet}

{#snippet loadingSpinner()}
	<Spinner />
{/snippet}

{#await userData}
	<Spinner />
{:then connections}
	{@const connectionMap = connections.connections.reduce(
		(map, c) => {
			map[c.platform] = c;
			return map;
		},
		{} as { [key in Platform]: UserConnection },
	)}
	{@const platforms = PLATFORMS.toSorted((a, b) => {
		const aConnected = !!connectionMap[a.id];
		const bConnected = !!connectionMap[b.id];
		return +bConnected - +aConnected;
	})}
	{@const numLinkableConnections = PLATFORMS.filter(
		(p) => p.linkingEnabled && connectionMap[p.id],
	).length}

	{#each platforms as platform}
		<li>
			<div class="platform">
				{@render platform.icon()}
				{#if connectionMap[platform.id]}
					<span>{platform.name} – {connectionMap[platform.id].platformDisplayName}</span>
				{:else}
					<span>{platform.name}</span>
				{/if}
			</div>
			{#if connectionMap[platform.id]}
				{@const loading =
					removeLoading === platformToValue(platform.id, connectionMap[platform.id].platformId)}
				<Button
					secondary
					disabled={!!(
						numLinkableConnections <= 1 &&
						platform.linkingEnabled &&
						connectionMap[platform.id]
					) || loading}
					icon={loading ? loadingSpinner : undefined}
					onclick={() =>
						removeUserConnection(
							connectionMap[platform.id].platform,
							connectionMap[platform.id].platformId,
						)}
				>
					{$t("labels.disconnect")}
				</Button>
			{:else}
				<Button
					primary
					disabled={!platform.linkingEnabled}
					href={platform.linkingEnabled ? platform.linkUrl : undefined}
				>
					{platform.linkingEnabled ? $t("labels.connect") : "Disabled"}
				</Button>
			{/if}
		</li>
	{/each}
{/await}

<style lang="scss">
	li {
		display: flex;
		justify-content: space-between;
		align-items: center;

		padding: 0.5rem 1rem;
		background-color: var(--bg-light);
		border-radius: 0.5rem;

		.platform {
			display: flex;
			align-items: center;
			gap: 1rem;

			font-size: 0.875rem;
			font-weight: 500;
		}
	}
</style>
