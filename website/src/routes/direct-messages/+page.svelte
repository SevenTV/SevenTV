<script lang="ts">
	import DirectMessages from "$/components/direct-messages/direct-messages.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("common.direct_messages")} - 7TV</title>
</svelte:head>

<DirectMessages />
