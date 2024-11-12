<script lang="ts">
	import Feature from "./feature.svelte";
	import { PUBLIC_DISCORD_LINK } from "$env/static/public";
	import weirdSmiley from "$assets/weird-smiley.webp?url";
	import cube from "$assets/cube.webp?url";
	import hand from "$assets/hand.webp?url";
	import hourglass from "$assets/hourglass.webp?url";
	import megaphone from "$assets/megaphone.webp?url";
	import battery from "$assets/battery.webp?url";
</script>

<section class="features">
	<h2>Why 7TV?</h2>
	<p>
		Our platform offers a variety of useful features that will enhance your chatting experience.
	</p>
	<div class="feature-grid">
		<Feature
			image={weirdSmiley}
			title="1000 Emote Slots"
			description="Everyone gets 1000 customizable channel emote slots, all for free. Keep all your emotes in one place."
			link="Browse Emotes"
			href="/emotes"
		/>
		<Feature
			image={cube}
			title="Emote Sets"
			description="Group emotes in customizable sets that can be shared with other users or quickly swapped onto your channel."
			link="Create an Emote Set"
			href="/emote-sets"
		/>
		<Feature
			image={hand}
			title="Custom Emote Names"
			description="Don't like the name given to an emote by its author? That's fine, you can change it for your channel only."
			link="Browse Emotes"
			href="/emotes"
		/>
		<Feature
			image={hourglass}
			title="Real-Time Updates"
			description="Changing emotes in your channel happens instantly, for all viewers. No F5 required, no more waiting."
			link="Download the Extension"
			href="#download"
		/>
		<Feature
			image={megaphone}
			title="Large Community"
			description="7TV serves 2,000,000+ daily unique users and has a library of over 1,000,000 public emotes."
			link="Join our Discord"
			href={PUBLIC_DISCORD_LINK}
		/>
		<Feature
			image={battery}
			title="Next-gen Formats"
			description="We use newer, more optimized image formats like WebP and AVIF, to reduce bandwidth usage."
			link="Download the Extension"
			href="#download"
		/>
	</div>
</section>

<style lang="scss">
	.features {
		text-align: center;
		font-size: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		align-items: center;

		h2 {
			font-family: "AKONY";
			font-size: 2.8rem;
			font-weight: 700;

			word-break: break-word;
		}

		p {
			font-size: 1.125rem;
			font-weight: 400;
			color: var(--text-light);
			line-height: 1.6em;
		}
	}

	.feature-grid {
		margin: 2.5rem 0;

		width: 100%;
		// 3 columns + 2 gaps
		// max-width: 80rem;
		max-width: min(80rem, 100%);

		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr));
		gap: 2.4rem;
		justify-content: center;
		align-items: center;
	}
</style>
