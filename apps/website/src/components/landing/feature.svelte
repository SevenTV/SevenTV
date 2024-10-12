<script lang="ts">
	export let image: string;
	export let title: string;
	export let description: string;
	export let link: string;
	export let href: string;
</script>

<div class="feature">
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

			img {
				height: 6.25rem;
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
</style>
