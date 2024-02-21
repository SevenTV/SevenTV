<script lang="ts">
	import { faEllipsisV, faRepeat } from "@fortawesome/pro-regular-svg-icons";
	import StoreSection from "./store-section.svelte";
	import Fa from "svelte-fa";
</script>

<StoreSection title="Monthly Paints">
    <div class="buttons" slot="header">
        <div class="renew-countdown">
            <Fa icon={faRepeat} size="1.2x" />
            <span>12 d</span>
        </div>
        <button class="button secondary square">
            <Fa icon={faEllipsisV} size="1.2x" fw />
        </button>
    </div>
    {#each Array(2) as _}
        <div class="paint">
            Paint
            <div class="circle"></div>
        </div>
    {/each}
</StoreSection>

<style lang="scss">
    .buttons {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .renew-countdown {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        white-space: nowrap;

        color: var(--text-lighter);
        font-size: 0.75rem;
    }

    .paint {
        padding: 0.5rem 1rem;
        background-color: var(--bg-light);

        display: flex;
        justify-content: space-between;
        align-items: center;

        color: var(--text-lighter);
        font-weight: 600;

        .circle {
            width: 1.5rem;
            height: 1.5rem;
            background-color: var(--secondary);
            border-radius: 50%;
        }
    }
</style>
