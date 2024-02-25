<script lang="ts">
	// different types of buttons:
	// - links looking like buttons
	// - real buttons with on:click
	//
	// different styles:
	// - icon only (square), text only, icon + text
	// - primary, secondary, no bg
	// - disabled, active, hover (focus)

	// This prop decides if this is a link looking like button or if it's a real button
	export let href: string | null = null;

	export let primary: boolean = false;
	export let secondary: boolean = false;
    
    export let big: boolean = false;

    export let hideOnMobile: boolean = false;
    export let hideOnDesktop: boolean = false;
</script>

{#if href}
	<a
        {href}
        {...$$restProps}
        class="button"
		class:primary
		class:secondary
        class:big
        class:has-text={$$slots.default}
		class:icon-only={!$$slots.default && ($$slots.icon || $$slots["icon-right"])}
		class:icon-left={$$slots.icon}
		class:icon-right={$$slots["icon-right"]}
        class:hide-on-mobile={hideOnMobile}
        class:hide-on-desktop={hideOnDesktop}
	>
		<slot name="icon" />
		<slot />
		<slot name="icon-right" />
	</a>
{:else}
	<button
		on:click
        {...$$restProps}
        class="button"
		class:primary
		class:secondary
        class:big
        class:has-text={$$slots.default}
		class:icon-only={!$$slots.default && ($$slots.icon || $$slots["icon-right"])}
		class:icon-left={$$slots.icon}
		class:icon-right={$$slots["icon-right"]}
        class:hide-on-mobile={hideOnMobile}
        class:hide-on-desktop={hideOnDesktop}
	>
		<slot name="icon" />
		<slot />
		<slot name="icon-right" />
	</button>
{/if}

<style lang="scss">
    a, button {
        cursor: pointer;
        font: inherit;
        border: none;
        color: var(--text);
        font-size: 0.875rem;
        font-weight: 600;
        padding: 0.5rem;
        border-radius: 0.5rem;
        transition: background-color 0.1s;
        white-space: nowrap;
        user-select: none;

        display: flex;
        align-items: center;
        gap: 0.5rem;

        &.has-text {
            padding: 0.5rem 1rem;

            &.icon-left {
                padding-left: 0.75rem;
            }

            &.icon-right {
                padding-right: 0.75rem;
            }
        }

        &.big {
            padding: 0.75rem;
            font-weight: 500;

            gap: 0.75rem;
        }
        
        &.primary {
            background-color: var(--secondary);

            &:disabled {
                background-color: var(--secondary-disabled);
            }
        }

        &:hover,
        &:focus-visible {
            text-decoration: none;
            background-color: var(--secondary-hover);
        }

        &:active {
            background-color: var(--secondary-active);
        }

        &:disabled {
            cursor: not-allowed;
        }

        &.secondary {
            background-color: var(--primary);

            &:hover,
            &:focus-visible {
                background-color: var(--primary-hover);
            }

            &:active {
                background-color: var(--primary-active);
            }

            &:disabled {
                background-color: var(--primary-disabled);
            }
        }
    }
</style>
