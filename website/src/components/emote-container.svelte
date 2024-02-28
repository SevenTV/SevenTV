<script lang="ts">
    export let scrollable = false;
    export let topMargin: number = 0;
</script>

<div class="emotes" style="--top-margin: {topMargin}rem" class:scrollable={scrollable}>
	<slot />
</div>

<style lang="scss">
    .emotes {
		margin-top: var(--top-margin);
		
        &.scrollable {
            overflow: auto;
        }

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(var(--emote-preview-size), 1fr));
		place-items: center;
		gap: 1rem;
	}

    @media screen and (max-width: 960px) {
		.emotes {
			gap: 0.5rem;
		}
	}
</style>
