<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import { CaretDown, Clock, DotsThreeVertical, Gift, Hourglass, Sparkle, Star } from "phosphor-svelte";

    export let subbed = false;
</script>

<StoreSection title="Your Subscription">
    <div class="buttons" slot="header">
        <button class="button secondary hide-on-mobile" class:sub-color={subbed} on:click={() => (subbed = true)}>
            <Star weight={subbed ? "fill" : "bold"} />
            <span>{subbed ? "Subscribed" : "Subscribe"}</span>
            <CaretDown />
        </button>
        <button class="button secondary icon-left icon-right hide-on-desktop" class:sub-color={subbed} on:click={() => (subbed = true)}>
            <Star weight={subbed ? "fill" : "bold"} />
            <CaretDown />
        </button>

        <button class="button secondary icon-left hide-on-mobile">
            <Gift />
            Gift
        </button>
        <button class="button secondary square hide-on-desktop">
            <Gift />
        </button>

        <button class="button secondary square">
            <DotsThreeVertical />
        </button>
    </div>
    <div class="sub-grid">
        <span class="key">Plan</span>
        <span class="value">
            <Sparkle />
            <span>Free</span>
        </span>

        <span class="key">Gifted By</span>
        <span class="value">
            <Gift />
            <span>N/A</span>
        </span>

        <span class="key">Sub Ends</span>
        <span class="value">
            <Hourglass />
            <span>N/A</span>
        </span>

        <span class="key">Previously Subscribed</span>
        <span class="value">
            <Clock />
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

    @media screen and (max-width: 960px) {
        .sub-grid {
            grid-template-rows: repeat(4, auto);
        }
    }
</style>
