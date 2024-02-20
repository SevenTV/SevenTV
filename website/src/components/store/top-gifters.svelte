<script lang="ts">
    import StoreSection from "./store-section.svelte";
	import Fa from "svelte-fa";
	import { faChevronLeft, faChevronRight } from "@fortawesome/pro-solid-svg-icons";
	import { faGift, faTrophy } from "@fortawesome/pro-regular-svg-icons";

    const topGifters = [
        {
            name: "Pokimane",
            amount: 215
        },
        {
            name: "CupOfEggy",
            amount: 164
        },
        {
            name: "forsen",
            amount: 121
        },
        {
            name: "ayyybubu",
            amount: 89
        },
        {
            name: "xQc",
            amount: 56
        },
    ]
</script>

<StoreSection title="Top Gifters">
    <div class="header" slot="header">
        <button class="button">
            <Fa icon={faChevronLeft} size="1.2x" />
        </button>
        <button class="button">
            <Fa icon={faChevronRight} size="1.2x" />
        </button>
    </div>
    <div class="grid">
        {#each topGifters as gifter, i}
            <span class="rank">{i + 1}.</span>
            <div class="profile-picture"></div>
            <span class="name">{gifter.name}</span>
            <span class="amount">
                <span class:gold={i === 0} class:silver={i === 1} class:bronze={i === 2}>
                    {#if i < 3}
                        <Fa icon={faTrophy} size="1.2x" />
                    {/if}
                </span>
                {gifter.amount}
            </span>
        {/each}
    </div>
    <button class="gift button secondary icon-left">
        <Fa icon={faGift} size="1.2x" />
        Gift
    </button>
</StoreSection>

<style lang="scss">
    .header {
        display: flex;
        gap: 1rem;
    }

    .grid {
        display: grid;
        grid-template-columns: auto auto 1fr auto;
        align-items: center;
        column-gap: 0.75rem;
        row-gap: 1rem;

        font-size: 0.875rem;
        font-weight: 500;

        & > .rank {
            color: var(--text-lighter);
        }

        & > .profile-picture {
            width: 1.5rem;
            height: 1.5rem;
            border-radius: 50%;
            background-color: var(--secondary);
        }

        & > .amount {
            display: flex;
            gap: 0.75rem;
            align-items: center;
            justify-content: flex-end;
        }
    }

    .gift.button {
        align-self: flex-end;
    }

    .gold {
        color: #eab631;
    }

    .silver {
        color: #c8c8c8;
    }

    .bronze {
        color: #c27924;
    }
</style>
