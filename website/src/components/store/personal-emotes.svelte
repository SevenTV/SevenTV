<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import Fa from "svelte-fa";
	import { faPenToSquare } from "@fortawesome/pro-regular-svg-icons";
</script>

<StoreSection title="Personal Emotes">
	<div class="container">
		<EmoteSetPreview bg="light" />
	</div>
	<button class="button edit secondary icon-left">
		<Fa icon={faPenToSquare} size="1.2x" />
		Edit
	</button>
</StoreSection>

<style lang="scss">
	.container {
		flex-grow: 1;

		display: flex;
		justify-content: center;
		align-items: center;
	}

	.button.edit {
		align-self: flex-end;
	}
</style>
