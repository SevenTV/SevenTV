<script lang="ts">
	import type { Moment } from "moment/min/moment-with-locales";

	let { date }: { date: Moment } = $props();
</script>

<span title={date.fromNow()}>{date.format("ll")}</span>
