<script lang="ts">
	import type { Moment } from "moment/min/moment-with-locales";

    export let date: Moment;
</script>

<span title={date.format("lll")}>{date.fromNow()}</span>
