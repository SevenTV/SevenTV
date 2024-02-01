<script lang="ts">
	import Fa from "svelte-fa";
	import { faChevronDown } from "@fortawesome/pro-solid-svg-icons";
	import mouseTrap from "$/lib/mouseTrap";
	import { fade } from "svelte/transition";

    type Option = { name: string; key: number; };

    export let options: Option[];
    export let selected: number = options[0].key;

    $: selectedName = options.find((v) => v.key === selected)?.name;

    let expanded = false;

    function toggle() {
		expanded = !expanded;
	}

	function close() {
		expanded = false;
	}
</script>

<button
	on:click={toggle}
	use:mouseTrap={close}
    class="button secondary active"
    class:expanded={expanded}
>
    {selectedName ?? "Select"}
    <Fa icon={faChevronDown} fw />
    {#if expanded}
        <div class="dropped" transition:fade={{ duration: 100 }}>
            {#each options as option}
                <button
                    class="button"
                    class:secondary={selected === option.key}
                    on:click={() => (selected = option.key)}
                >
                    {option.name}
                </button>
            {/each}
        </div>
    {/if}
</button>

<style lang="scss">
    .button.active {
        position: relative;
        width: 100%;
        justify-content: space-between;
        border: transparent 1px solid;
        border-bottom: none;
        background-color: var(--secondary);

        &.expanded {
            border-color: var(--border);
            border-bottom-left-radius: 0;
            border-bottom-right-radius: 0;

            & > .dropped {
                border-top-left-radius: 0;
                border-top-right-radius: 0;
            }
        }
    }

    .dropped {
        z-index: 1;

        position: absolute;
        top: 100%;
        left: -1px;
        right: -1px;
        margin: 0;
        padding: 0;
        border: var(--border) 1px solid;
        border-top: none;
        border-radius: 0.5rem;

        background-color: var(--bg-medium);
        box-shadow: 4px 4px 0px rgba(0, 0, 0, 0.25);

        & > button {
            border-radius: 0;
            width: 100%;
            padding: 0.5rem 1rem;

            &:last-child {
                border-bottom-left-radius: 0.5rem;
                border-bottom-right-radius: 0.5rem;
            }
        }
    }
</style>
