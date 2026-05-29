<script lang="ts">
	import EmotesUploaded from "./global/emotes-uploaded.svelte";
	import EmotesSent from "./global/emotes-sent.svelte";
	import TopChannels from "$/components/recap/global/top-channels.svelte";
	import EmoteSets from "./global/emote-sets.svelte";
	import ActiveMods from "./global/active-mods.svelte";
	import EmotesUploaders from "./global/emotes-uploaders.svelte";
	import EmoteSetsCreated from "./global/emote-sets-created.svelte";
	import type { Paint, Badge } from "$/gql/graphql";
	import TopCosmetics from "./global/top-cosmetics.svelte";
	import TopTags from "./global/top-tags.svelte";

	interface Props {
		potatData: any;
		cosmetics: { paints: Paint[]; badges: Badge[] };
	}

	let { potatData, cosmetics }: Props = $props();
	const topChannelsData = potatData.data[0].global.channels || [];
	const emotesSentData = potatData.data[0].global.top_used || [];
	const emoteSentCount = potatData.data[0].global.sum_used || 0;
</script>

<EmotesUploaded />
<EmotesSent {emotesSentData} {emoteSentCount} />
<TopChannels {topChannelsData} />
<EmoteSets />
<ActiveMods />
{#if cosmetics}
	<TopCosmetics
		{cosmetics}
	/>
{/if}
<EmotesUploaders />
<EmoteSetsCreated />
<TopTags />
