<script lang="ts">
	import { dateFormat, timeFormat } from "$/lib/layout";

	import type { Moment } from "moment/min/moment-with-locales";

	let { date, keepRelative = false }: { date: Moment; keepRelative?: boolean } = $props();

	const exactDate = $dateFormat.endsWith("exact") && !keepRelative;
	const is24h = $timeFormat.endsWith("24h");

	let exactTimeString = $state("MMM D, YYYY h:mm:ss.SSS A");
	if (is24h) exactTimeString = "MMM D, YYYY H:mm:ss.SSS";
</script>

<span title={exactDate ? date.fromNow() : date.format("lll")}
	>{exactDate ? date.format(exactTimeString) : date.fromNow()}</span
>
