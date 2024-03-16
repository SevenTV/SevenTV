<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import TwitchLogo from "$/components/icons/twitch-logo.svelte";
	import YoutubeLogo from "$/components/icons/youtube-logo.svelte";
	import XTwitterLogo from "$/components/icons/x-twitter-logo.svelte";
	import { t } from "svelte-i18n";
</script>

<div class="layout">
	<h2>{$t("common.connections")}</h2>
	<div class="link-list">
		<Button href="https://twitch.tv/ayyybubu" target="_blank" big>
			<TwitchLogo slot="icon" />
			<span>ayyybubu</span>
		</Button>
		<Button href="https://youtube.com/channel/bubutv" target="_blank" big>
			<YoutubeLogo slot="icon" />
			<span>bubutv</span>
		</Button>
		<Button href="https://twitter.com/tweetbubu" target="_blank" big>
			<XTwitterLogo slot="icon" />
			<span>tweetbubu</span>
		</Button>
	</div>
	<h2>{$t("common.editors")}</h2>
	<div class="link-list">
		<ChannelPreview big />
		<ChannelPreview big />
	</div>
</div>

<style lang="scss">
	.layout {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h2 {
		font-size: 1rem;
		font-weight: 600;
	}

	.link-list {
		display: flex;
		flex-direction: column;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}
</style>
