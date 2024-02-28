<script lang="ts">
	import Button from "../button.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretLeft, CaretRight, Crown, Ticket } from "phosphor-svelte";
</script>

<StoreSection title="Global Emote Raffle">
	<span class="header" slot="header">Dec 2023</span>
	<div class="carousel">
		<Button>
			<CaretLeft slot="icon" />
		</Button>
		<EmotePreview bg="light" />
		<Button>
			<CaretRight slot="icon" />
		</Button>
	</div>
	<div class="footer">
		<div class="winner">
			<div class="profile-picture"></div>
			<span>Pokimane</span>
			<Crown color="#aa71ff" />
		</div>
		<Button secondary>
			<Ticket slot="icon" />
			Enter
		</Button>
	</div>
</StoreSection>

<style lang="scss">
	.header {
		color: var(--text-light);
		font-size: 0.875rem;
		font-weight: 500;
	}

	.carousel {
		flex-grow: 1;

		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.winner {
		display: flex;
		align-items: center;
		gap: 0.75rem;

		font-size: 0.875rem;
		font-weight: 500;

		.profile-picture {
			width: 1.5rem;
			height: 1.5rem;
			background-color: var(--secondary);
			border-radius: 50%;
		}
	}
</style>
