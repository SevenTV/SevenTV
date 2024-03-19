<script lang="ts">
	import Dialog from "./dialog.svelte";
    import type { DialogMode } from "./dialog.svelte";

    export let mode: DialogMode;
</script>

<Dialog bind:mode>
    <h1>Emote queue</h1>
    <hr />
</Dialog>

<style lang="scss">
    
</style>
