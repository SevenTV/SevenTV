<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import { NotePencil } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";
</script>

<StoreSection title={$t("common.personal_emotes")}>
	<div class="container">
		<EmoteSetPreview bg="light" />
	</div>
	<Button secondary style="align-self: flex-end">
		<NotePencil slot="icon" />
		{$t("labels.edit")}
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
