<script lang="ts">
	export let title: string;
	export let subtitle: string;
	export let gradientColor: string = "#9227cf";
</script>

<section class="banner" style="--gradient-color: {gradientColor}">
	<span>
		<h1>{title}</h1>
		<p>{subtitle}</p>
	</span>
	<slot />
</section>

<style lang="scss">
	.banner {
		position: relative;
		z-index: 0;

		&::before {
			content: "";
			position: absolute;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			border-radius: 0.5rem;

			z-index: -1;

			background: radial-gradient(
				100% 100% at 50% 0%,
				#ddc4fd 0%,
				var(--gradient-color) 33%,
				#3f188b 64%,
				#1f113e 80%,
				#0f0f0f 100%
			),
			var(--bg-medium);

			mask-image: radial-gradient(
				200% 100% at 50% 0%,
				rgba(white, 1) 0%,
				rgba(white, 0.7) 33%,
				rgba(white, 0.2) 66%,
				rgba(white, 0) 100%
			);
			mask-size: 100% 400%;
			animation: fade-in 0.5s linear forwards;
		}
		background-color: var(--bg-medium);

		min-width: 12rem;
		min-height: 11.25rem;
		padding: 1.25rem 2.5rem;

		display: flex;
		justify-content: space-between;
		align-items: center;

		h1 {
			font-size: 1.875rem;
			font-weight: 700;
		}

		p {
			font-weight: 300;
		}
	}

	@keyframes fade-in {
		from {
			mask-position: 0% 100%;
		}
		to {
			mask-position: 0% 0%;
		}
	}

	@media screen and (max-width: 768px) {
		.banner {
			padding: 1rem;
		}
	}
</style>
