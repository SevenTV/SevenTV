<script lang="ts">
	import { Gear } from "phosphor-svelte";
	import DropDown from "../drop-down.svelte";
	import Button from "../input/button.svelte";
	import Select from "../input/select.svelte";
	import Toggle from "../input/toggle.svelte";

    export let actionsPosition: "left" | "right" = "left";
    export let buttonOptions: {
        merge: boolean;
        delete: boolean;
        unlist: boolean;
        approve: boolean;
    };
</script>

<th class="shrink">
    <div class="buttons">
        <DropDown align={actionsPosition}>
            <Button>
                <Gear slot="icon" />
            </Button>
            <div slot="dropdown" class="dropdown">
                <Select bind:selected={actionsPosition} options={[{value: "left", label: "Left"}, {value: "right", label: "Right"}]} grow />
                <Toggle bind:value={buttonOptions.merge}>
                    Show Merge
                </Toggle>
                <Toggle bind:value={buttonOptions.delete}>
                    Show Delete
                </Toggle>
                <Toggle bind:value={buttonOptions.unlist}>
                    Show Unlist
                </Toggle>
                <Toggle bind:value={buttonOptions.approve}>
                    Show Approve
                </Toggle>
            </div>
        </DropDown>
    </div>
</th>

<style lang="scss">
    .buttons {
		display: flex;
		align-items: center;
	}

    .dropdown {
		padding: 1rem;
		min-width: 11rem;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		color: var(--text);
		font-size: 0.875rem;
	}
</style>
