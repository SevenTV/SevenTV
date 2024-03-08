<script lang="ts">
	import type { Moment } from "moment";

	export let date: Moment;
</script>

<span class="date" title={date.fromNow()}>{date.format("ll")}</span>
