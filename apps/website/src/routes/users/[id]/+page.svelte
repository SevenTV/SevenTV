<script lang="ts">
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import { SortBy } from "$/gql/graphql";
</script>

<EmoteLoader options={{query: null, sortBy: SortBy.TopDaily, tags: [], filters: {}}} />
