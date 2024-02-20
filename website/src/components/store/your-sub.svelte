<script lang="ts">
    import Fa from "svelte-fa";
    import { faStar as faStarSolid } from "@fortawesome/pro-solid-svg-icons";
	import { faChevronDown, faClock, faEllipsisV, faGift, faHourglass, faSparkle, faStar as faStarRegular } from "@fortawesome/pro-regular-svg-icons";
	import StoreSection from "./store-section.svelte";
	import HideOn from "../hide-on.svelte";

    export let subbed = false;
</script>

<StoreSection title="Your Subscription">
    <div class="buttons" slot="header">
        <button class="button secondary icon-left icon-right" class:sub-color={subbed} on:click={() => (subbed = true)}>
            <Fa icon={subbed ? faStarSolid : faStarRegular} size="1.2x" />
            <span class="hide-on-mobile">{subbed ? "Subscribed" : "Subscribe"}</span>
            <Fa icon={faChevronDown} size="1.2x" />
        </button>
        <HideOn mobile>
            <button class="button secondary icon-left">
                <Fa icon={faGift} size="1.2x" />
                Gift
            </button>
        </HideOn>
        <HideOn desktop>
            <button class="button secondary">
                <Fa icon={faGift} size="1.2x" />
            </button>
        </HideOn>
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

    .sub-color {
        color: var(--subscriber-accent);
    }

    .sub-grid {
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
