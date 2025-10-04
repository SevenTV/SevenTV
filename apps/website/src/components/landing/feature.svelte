<script lang="ts">
	interface Props {
		image: string;
		title: string;
		description: string;
		link: string;
		href: string;
		animationDelay?: number;
	}

	let { image, title, description, link, href, animationDelay = 0 }: Props = $props();
</script>

<div class="feature" style="--animation-delay: {animationDelay}s">
	<div class="background"></div>
	<div class="content">
		<img src={image} alt={title} />
		<h3>{title}</h3>
		<p>{description}</p>
		<a {href}>{link} -></a>
	</div>
</div>

<style lang="scss">
	.feature {
		height: 100%;

		position: relative;
		z-index: 0;
		transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);

		&:hover {
			.content {
				background-color: rgba(121, 121, 121, 0.08);
				border-color: rgba(255, 255, 255, 0.2);
				box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
				
				img {
					animation: float 4s linear infinite !important;
					animation-fill-mode: both !important;
					will-change: transform;
					backface-visibility: hidden;
					transform: translateZ(0);
				}
			}
		}

		.background {
			position: absolute;
			z-index: -1;
			top: 10%;
			left: 10%;
			right: 10%;
			bottom: 10%;
			background: radial-gradient(circle at bottom, rgba(56, 0, 176, 0.3), rgba(0, 215, 244, 0.15));
			filter: blur(50px);
		}

		.content {
			height: 100%;

			display: flex;
			flex-direction: column;
			justify-content: space-between;
			align-items: center;
			gap: 1rem;

			backdrop-filter: blur(50px);
			padding: 2rem 1.5rem;
			border-radius: 1.5rem;
			border: 1px solid var(--border-active);
			background-color: rgba(121, 121, 121, 0.04);
			
			transition: 
				background-color 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94),
				border-color 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94),
				box-shadow 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);

			img {
				height: 6.25rem;
				transform-origin: center center;
				transition: transform 0.3s ease-in-out;
			}

			h3 {
				font-size: 2rem;
				font-weight: 700;

				margin-top: 1rem;
			}

			p {
				color: var(--text-light);
				font-size: 1.125rem;
				font-weight: 400;
				line-height: 1.8rem;

				margin-bottom: 1rem;
			}

			a {
				color: var(--store);
				font-size: 1rem;
				font-weight: 600;
			}
		}
	}

	@keyframes float {
		0% {
			transform: translate3d(0, 0px, 0) rotate(0deg);
			animation-timing-function: cubic-bezier(0.445, 0.05, 0.55, 0.95);
		}
		50% {
			transform: translate3d(0, -12px, 0) rotate(0deg);
			animation-timing-function: cubic-bezier(0.445, 0.05, 0.55, 0.95);
		}
		100% {
			transform: translate3d(0, 0px, 0) rotate(0deg);
		}
	}
</style>
