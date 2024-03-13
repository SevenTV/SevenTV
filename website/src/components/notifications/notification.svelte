<script lang="ts">
	import moment from "moment";
	import Button from "../input/button.svelte";
</script>

<div class="notification">
	<div class="placeholder"></div>
	<span class="event">Your emote AlienPls has been Approved for Public Listing.</span>
	<span class="details">{moment("1970-01-01T00:00:00").fromNow()} · System</span>
	<div class="buttons">
		<Button primary>Accept</Button>
		<Button secondary>Deny</Button>
	</div>
</div>

<style lang="scss">
	.notification {
		display: grid;
		grid-template-columns: auto 1fr;
		column-gap: 1rem;
		row-gap: 0.5rem;
		align-items: center;
	}

	.placeholder {
		grid-row: span 2;

		width: 3rem;
		height: 3rem;
		border-radius: 50%;
		background-color: var(--secondary);
	}

	.event {
		font-size: 0.875rem;
	}

	.details {
		color: var(--text-light);
		font-size: 0.75rem;
	}

	.buttons {
		grid-column: 2;

		display: flex;
		gap: 0.5rem;
	}
</style>
