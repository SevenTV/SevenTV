<script lang="ts">
	import { writable } from "svelte/store";

	let notifications = writable<{ id: number; message: string }[]>([]);
</script>

<div class="notification-container">
	{#each $notifications as notification (notification.id)}
		<div class="notification">
			{notification.message}
		</div>
	{/each}
</div>

<style lang="scss">
	.notification-container {
		position: fixed;
		bottom: 1rem;
		right: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		z-index: 1000;
	}

	.notification {
		background-color: #333;
		color: #fff;
		padding: 1rem 1.5rem;
		border-radius: 0.75rem;
		box-shadow: 0 3px 7px rgba(0, 0, 0, 0.3);
		font-size: 1.25rem;
		animation: fade-in-out 99s ease-in-out;
	}

	@keyframes fade-in-out {
		0% {
			opacity: 0;
			transform: translateY(10px);
		}
		10%,
		90% {
			opacity: 1;
			transform: translateY(0);
		}
		100% {
			opacity: 0;
			transform: translateY(10px);
		}
	}
</style>
