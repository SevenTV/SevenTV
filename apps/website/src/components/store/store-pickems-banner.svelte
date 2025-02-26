<script lang="ts">
	import type { Snippet } from "svelte";
	import csMoneyIcon from "$assets/csmoneyiconpickems.png?url";

	interface Props {
		title: string;
		title2: string;
		gradientColor?: string;
		children?: Snippet;
	}

	let { title, title2, gradientColor = "#9227cf", children }: Props = $props();
</script>

<section class="banner" style="--gradient-color: {gradientColor}">
	<span>
		<h1>{title}</h1>
		<h2>{title2}</h2>
	</span>
	<img src={csMoneyIcon} alt="CS Money Icon" class="csmoney-banner-image" />
	{@render children?.()}
</section>

<style lang="scss">
	.csmoney-banner-image {
		position: absolute;
		top: 1rem;
		right: 1rem;
		width: 9rem;
		height: auto;
		@media screen and (max-width: 600px) {
			display: none;
		}
	}
	.banner {
		position: relative;
		z-index: 0;

		color: white;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		min-width: 12rem;
		min-height: 11.25rem;
		padding: 1.25rem 2.5rem;

		display: flex;
		justify-content: space-between;
		align-items: center;

		&::before {
			content: "";
			position: absolute;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			border-radius: 0.5rem;

			z-index: -1;

			background: radial-gradient(186.65% 172.97% at 50% 163.29%, #e88400 0%, #4a00ab 100%),
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

		h1 {
			opacity: 0.9;
			background: linear-gradient(0deg, #fff 24.23%, rgba(255, 255, 255, 0.61) 100%);
			background-clip: text;
			-webkit-background-clip: text;
			-webkit-text-fill-color: transparent;
			font-family: Inter;
			font-size: 1.5rem;
		}

		h2 {
			opacity: 0.9;
			background: linear-gradient(0deg, #fff 24.23%, rgba(255, 255, 255, 0.61) 100%);
			background-clip: text;
			-webkit-background-clip: text;
			-webkit-text-fill-color: transparent;
			font-weight: 600;
			font-size: 2.5rem;
			font-family: Inter, serif;
		}
	}

	@keyframes fade-in {
		from {
			mask-position: 0 100%;
		}
		to {
			mask-position: 0 0;
		}
	}

	@media screen and (max-width: 768px) {
		.banner h2 {
			font-size: 2rem;
		}
	}

	@media screen and (max-width: 500px) {

		.banner{
			justify-content: center;
			text-align: center;
		}
		.banner h1{
			font-size: 1.3rem;
			text-align: center;
		}
		.banner h2 {
			font-size: 1.6rem;
			text-align: center;
		}
	}
</style>
