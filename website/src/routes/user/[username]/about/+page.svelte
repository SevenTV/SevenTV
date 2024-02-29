<script lang="ts">
	import Button from "$/components/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import { TwitchLogo, TwitterLogo, YoutubeLogo } from "phosphor-svelte";
</script>

<div class="layout">
	<h2>Connections</h2>
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
			<TwitterLogo slot="icon" />
			<span>tweetbubu</span>
		</Button>
	</div>
	<h2>Editors</h2>
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
