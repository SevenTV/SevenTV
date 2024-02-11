<script lang="ts">
	import Fa from "svelte-fa";
    import StoreSection from "./store-section.svelte";
	import { faEllipsisV } from "@fortawesome/pro-regular-svg-icons";

    export let percentage: number = 40;
</script>

<StoreSection>
    <div class="container">
        <div class="progress-circle" style="--percentage: {percentage}%">
            {percentage}%
        </div>
        <div class="info">
            <div class="header">
                <h2>Badge Progress</h2>
                <button class="button secondary">
                    <Fa icon={faEllipsisV} size="1.2x" />
                </button>
            </div>
            <div class="badges">
                <div class="badge">
                    <div class="placeholder"></div>
                    <span>1 Year</span>
                </div>
                <div class="bar-container">
                    <span class="countdown">23 days left</span>
                    <div class="bar"></div>
                </div>
                <div class="badge">
                    <div class="placeholder"></div>
                    <span>1.5 Years</span>
                </div>
            </div>
        </div>
    </div>
</StoreSection>

<style lang="scss">
    .container {
        display: flex;
        gap: 1.25rem;
    }

    .progress-circle {
        width: 8rem;
        height: 8rem;

        font-size: 1.25rem;
        font-weight: 700;

        display: flex;
        justify-content: center;
        align-items: center;

        border-radius: 50%;
        background-color: var(--secondary);
        z-index: 1;
        position: relative;

        &::before {
            content: "";
            position: absolute;
            border-radius: 50%;
            width: 8rem;
            height: 8rem;
            background: linear-gradient(0deg, #5D25FE, #FF36F7);
            -webkit-mask-image: conic-gradient(white 0%, white var(--percentage), transparent var(--percentage), transparent 100%);
            mask-image: conic-gradient(white 0%, white var(--percentage), transparent var(--percentage), transparent 100%);
            z-index: -2;
        }

        &::after {
            content: "";
            position: absolute;
            border-radius: 50%;
            width: 6rem;
            height: 6rem;
            background-color: var(--bg-medium);
            z-index: -1;
        }
    }

    .info {
        flex-grow: 1;

        display: flex;
        flex-direction: column;
        justify-content: space-between;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;

        h2 {
            font-size: 1rem;
            font-weight: 600;
        }
    }

    .badges {
        display: flex;
        gap: 0.5rem;

        color: var(--text-lighter);
        font-size: 0.75rem;
        font-weight: 600;

        .badge {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 0.5rem;

            & > .placeholder {
                width: 2.25rem;
                height: 2.25rem;
                background-color: var(--secondary);
                border-radius: 0.5rem;
            }
        }

        .bar-container {
            flex-grow: 1;
            height: 2.25rem;
            position: relative;

            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;

            .countdown {
                position: absolute;
                top: -0.5rem;
            }

            .bar {
                align-self: stretch;

                height: 0.25rem;
                background-color: var(--secondary);
                border-radius: 0.125rem;
            }
        }
    }
</style>
