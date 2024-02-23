<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import { PencilSimple } from "phosphor-svelte";
</script>

<StoreSection title="Personal Emotes">
	<div class="container">
		<EmoteSetPreview bg="light" />
	</div>
	<button class="button edit secondary icon-left">
		<PencilSimple />
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
