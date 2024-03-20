<script lang="ts">
	import EmoteTicketsButtonOptions from "./emote-tickets-button-options.svelte";

    export let actionsPosition: "left" | "right" = "left";
    export let buttonOptions: {
        merge: boolean;
        delete: boolean;
        unlist: boolean;
        approve: boolean;
    };
</script>

<th class="shrink">
    <div class="buttons">
        <EmoteTicketsButtonOptions bind:buttonOptions bind:actionsPosition />
    </div>
</th>

<style lang="scss">
    .buttons {
		display: flex;
		align-items: center;
	}
</style>
