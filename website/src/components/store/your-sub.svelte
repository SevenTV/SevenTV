<script lang="ts">
    import Fa from "svelte-fa";
	import { faClock, faEllipsisV, faGift, faHourglass, faSparkle, faStar } from "@fortawesome/pro-regular-svg-icons";
	import StoreSection from "./store-section.svelte";

    export let subbed = false;
</script>

<StoreSection title="Your Subscription">
    <div class="buttons" slot="header">
        <button class="button secondary icon-left" on:click={() => (subbed = true)}>
            <Fa icon={faStar} size="1.2x" />
            Subscribe
        </button>
        <button class="button secondary icon-left">
            <Fa icon={faGift} size="1.2x" />
            Gift
        </button>
        <button class="button secondary">
            <Fa icon={faEllipsisV} size="1.2x" />
        </button>
    </div>
    <div class="sub-grid">
        <span class="key">Plan</span>
        <span class="value">
            <Fa icon={faSparkle} size="1.2x" />
            <span>Free</span>
        </span>

        <span class="key">Gifted By</span>
        <span class="value">
            <Fa icon={faGift} size="1.2x" />
            <span>N/A</span>
        </span>

        <span class="key">Sub Ends</span>
        <span class="value">
            <Fa icon={faHourglass} size="1.2x" />
            <span>N/A</span>
        </span>

        <span class="key">Previously Subscribed</span>
        <span class="value">
            <Fa icon={faClock} size="1.2x" />
            <span>0 days</span>
        </span>
    </div>
</StoreSection>

<style lang="scss">
    .buttons {
        display: flex;
        gap: 0.5rem;
    }

    .sub-grid {
        margin-top: 1rem;
        padding: 0.75rem;

        display: grid;
        grid-template-rows: repeat(2, auto);
        grid-auto-flow: column;
        row-gap: 0.5rem;
        column-gap: 2rem;

        .key {
            color: var(--text-lighter);
            font-size: 0.75rem;
            font-weight: 500;
        }

        .value {
            display: flex;
            gap: 0.5rem;
            align-items: center;

            color: var(--subscriber-accent);

            span {
                color: var(--text);
                font-size: 0.875rem;
                font-weight: 600;
            }
        }
    }
</style>
