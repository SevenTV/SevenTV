<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import { NotePencil } from "phosphor-svelte";
	import Button from "../button.svelte";
</script>

<StoreSection title="Personal Emotes">
	<div class="container">
		<EmoteSetPreview bg="light" />
	</div>
	<Button secondary style="align-self: flex-end">
		<NotePencil slot="icon" />
		Edit
	</Button>
</StoreSection>

<style lang="scss">
	.container {
		flex-grow: 1;

		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
