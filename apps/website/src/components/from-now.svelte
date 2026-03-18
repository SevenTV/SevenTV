<script lang="ts">
	import { dateFormat } from "$/lib/layout";

	import type { Moment } from "moment/min/moment-with-locales";

	const exactDate = $dateFormat.endsWith("exact");

	let { date }: { date: Moment } = $props();
</script>

<span title={exactDate ? date.fromNow() : date.format("lll")}
	>{exactDate ? date.format("MMM D, YYYY h:mm:ss.SSS A") : date.fromNow()}</span
>
