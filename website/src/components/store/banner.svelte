<script lang="ts">
	import {
		faBadge,
		faBrush,
		faSmilePlus,
		faTicket,
		faUserCircle,
	} from "@fortawesome/pro-regular-svg-icons";
	import Fa from "svelte-fa";

	export let title: string;
	export let subtitle: string;
</script>

<section class="banner">
	<span>
		<h1>{title}</h1>
		<p>{subtitle}</p>
	</span>
	<div class="icons hide-on-mobile">
		<Fa icon={faBrush} size="1.5x" />
		<Fa icon={faUserCircle} size="1.5x" />
		<Fa icon={faBadge} size="1.5x" />
		<Fa icon={faTicket} size="1.5x" />
		<Fa icon={faSmilePlus} size="1.5x" />
	</div>
</section>

<style lang="scss">
	.banner {
		border-radius: 0.5rem;
		background: radial-gradient(
				100% 100% at 50% 0%,
				#ddc4fd 0%,
				#9227cf 33.33%,
				#3f188b 64.58%,
				#1f113e 80.73%,
				#0e0c13 93.75%
			),
			var(--bg-medium);
		border-radius: 0.5rem;

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

		.icons {
			padding: 0 2.75rem;

			display: flex;
			gap: 3.5rem;
			flex-wrap: wrap;
			align-items: center;
			justify-content: center;
		}
	}

	@media screen and (max-width: 768px) {
		.banner {
			padding: 1rem;
		}
	}
</style>
