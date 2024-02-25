<script lang="ts">
	import ChannelPreview from "$/components/channel-preview.svelte";
	import SearchBar from "$/components/nav/search-bar.svelte";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
</script>

<div class="suggested-channels">
	<div class="navigation">
		<SearchBar />
		<div class="buttons">
			<button class="button square" disabled>
				<CaretLeft />
			</button>
			<button class="button square">
				<CaretRight />
			</button>
		</div>
	</div>
	<div class="channels">
		{#each Array(20) as _, i}
			<ChannelPreview index={i} user="channel{i}" bg="light" />
		{/each}
	</div>
</div>

<style lang="scss">
	.suggested-channels {
		margin-top: 1rem;

		.navigation {
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 1rem;

			.buttons {
				display: flex;

				button[disabled],
				.button:disabled {
					color: var(--text-light);
				}
			}
		}

		.channels {
			margin-top: 1.5rem;

			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
			justify-content: center;
			gap: 1rem;
		}
	}

	@media screen and (max-width: 960px) {
		.suggested-channels {
			.channels {
				margin-bottom: 0;
			}
		}
	}
</style>
