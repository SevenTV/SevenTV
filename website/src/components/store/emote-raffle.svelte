<script lang="ts">
    import EmotePreview from "../emote-preview.svelte";
    import StoreSection from "./store-section.svelte";
	import { CaretLeft, CaretRight, Crown, Ticket } from "phosphor-svelte";
</script>

<StoreSection title="Global Emote Raffle">
    <span class="header" slot="header">Dec 2023</span>
    <div class="carousel">
        <button class="button square">
            <CaretLeft />
        </button>
        <EmotePreview bg="light" />
        <button class="button square">
            <CaretRight />
        </button>
    </div>
    <div class="footer">
        <div class="winner">
            <div class="profile-picture"></div>
            <span>Pokimane</span>
            <Crown color="#aa71ff" />
        </div>
        <button class="button icon-left secondary">
            <Ticket />
            Enter
        </button>
    </div>
</StoreSection>

<style lang="scss">
    .header {
        color: var(--text-lighter);
        font-size: 0.875rem;
        font-weight: 500;
    }

    .carousel {
        flex-grow: 1;

        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .winner {
        display: flex;
        align-items: center;
        gap: 0.75rem;

        font-size: 0.875rem;
        font-weight: 500;

        .profile-picture {
            width: 1.5rem;
            height: 1.5rem;
            background-color: var(--secondary);
            border-radius: 50%;
        }
    }
</style>
