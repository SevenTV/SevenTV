<script lang="ts">
	import { ArrowsMerge, Check, EyeSlash, Trash } from "phosphor-svelte";
	import Button from "../input/button.svelte";

    export let buttonOptions: {
        merge: boolean;
        delete: boolean;
        unlist: boolean;
        approve: boolean;
    };
</script>
<td class="shrink">
    <div class="buttons">
        {#if buttonOptions.merge}
            <Button>
                <ArrowsMerge
                    slot="icon"
                    style="transform: rotate(-90deg)"
                    color="var(--admin-merge)"
                />
            </Button>
        {/if}
        {#if buttonOptions.delete}
            <Button>
                <Trash slot="icon" color="var(--danger)" />
            </Button>
        {/if}
        {#if buttonOptions.unlist}
            <Button>
                <EyeSlash slot="icon" color="var(--admin-unlist)" />
            </Button>
        {/if}
        {#if buttonOptions.approve}
            <Button>
                <Check slot="icon" color="var(--admin-approve)" />
            </Button>
        {/if}
    </div>
</td>

<style lang="scss">
    .buttons {
		display: flex;
		align-items: center;
	}
</style>
