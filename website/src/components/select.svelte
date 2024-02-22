<script lang="ts">
	import Fa from "svelte-fa";
	import { faChevronDown } from "@fortawesome/pro-solid-svg-icons";
	import mouseTrap from "$/lib/mouseTrap";
	import { fade } from "svelte/transition";

    export let options: string[];
    export let selected: string | null = options[0] ?? null;
    export let grow: boolean = false;

    let expanded = false;

    function toggle() {
        console.log("toggle");
		expanded = !expanded;
	}

	function close() {
		expanded = false;
	}
</script>

<button
	on:click={toggle}
	use:mouseTrap={close}
    class="button secondary select"
    class:grow={grow}
    class:expanded={expanded}
    tabindex="-1"
>
    <select bind:value={selected} on:click={toggle} on:keypress={toggle}>
        {#each options as option}
            <option value={option}>
                {option}
            </option>
        {/each}
    </select>
    {selected ?? "Select"}
    <Fa icon={faChevronDown} fw />
    {#if expanded}
        <div class="dropped" transition:fade={{ duration: 100 }}>
            {#each options as option}
                <button
                    class="button"
                    class:secondary={selected === option}
                    on:click={() => (selected = option)}
                >
                    {option}
                </button>
            {/each}
        </div>
    {/if}
</button>

<style lang="scss">
    select {
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
        outline: none;
        margin: 0;
        padding: 0;
        border: none;
        width: 0;

        display: inline;
        clip: rect(0 0 0 0);
        clip-path: inset(50%);
        height: 1px;
        overflow: hidden;
        position: absolute;
        white-space: nowrap;
        width: 1px;
    }

    .button.select {
        position: relative;
        justify-content: space-between;
        border: transparent 1px solid;
        background-color: var(--secondary);

        &:focus-within {
            border-color: var(--primary);
        }

        &.grow {
            width: 100%;
            flex-grow: 1;
        }

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
