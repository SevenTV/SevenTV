<script lang="ts">
	import TwitchLogo from "$/components/icons/twitch-logo.svelte";
	import YoutubeLogo from "$/components/icons/youtube-logo.svelte";
	import DiscordLogo from "$/components/icons/discord-logo.svelte";
	import Select, { type Option } from "$/components/input/select.svelte";
	import KickLogo from "$/components/icons/kick-logo.svelte";
	import type { Platform, User, UserConnection } from "$/gql/graphql";
	import type { Snippet } from "svelte";
	import Spinner from "$/components/spinner.svelte";
	import { setMainConnection } from "$/lib/userMutations";
	import { user } from "$/lib/auth";
	import { platformToValue, valueToPlatform } from "$/lib/utils";

	let { userData = $bindable() }: { userData: Promise<User> } = $props();

	const CONNECTION_ICONS: { [key in Platform]: Snippet } = {
		DISCORD: discordLogo,
		TWITCH: twitchLogo,
		GOOGLE: youtubeLogo,
		KICK: kickLogo,
	};

	let connectionOptions = $derived(
		userData.then((data) =>
			data?.connections.map((c) => {
				return {
					value: platformToValue(c.platform, c.platformId),
					label: c.platformDisplayName,
					icon: CONNECTION_ICONS[c.platform],
				} as Option;
			}),
		),
	);

	$effect(() => {
		userData.then((data) => {
			if (data?.mainConnection) {
				originalMainConnection = platformToValue(
					data.mainConnection.platform,
					data.mainConnection.platformId,
				);
				mainConnection = originalMainConnection;
			}
		});
	});

	let originalMainConnection: string;
	let mainConnection = $state<string>();

	$effect(() => {
		if (mainConnection && mainConnection !== originalMainConnection) {
			const { platform, platformId } = valueToPlatform(mainConnection);

			const promise = setMainConnection($user!.id, platform, platformId);
			promise.then((data) => {
				$user = data;
			});

			userData = promise;
		}
	});
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
{#await connectionOptions}
	<Select
		options={[{ value: "", label: "", icon: loadingSpinner }]}
		selected=""
		style="align-self: flex-start"
		disabled
	/>
{:then connectionOptions}
	{#if connectionOptions}
		<Select
			options={connectionOptions}
			bind:selected={mainConnection}
			style="align-self: flex-start"
		/>
	{/if}
{/await}
