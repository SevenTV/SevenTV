<script lang="ts">
	import Feature from "./feature.svelte";
	import { PUBLIC_DISCORD_LINK } from "$env/static/public";
	import weirdSmiley from "$assets/weird-smiley.webp?url";
	import cube from "$assets/cube.webp?url";
	import hand from "$assets/hand.webp?url";
	import hourglass from "$assets/hourglass.webp?url";
	import megaphone from "$assets/megaphone.webp?url";
	import battery from "$assets/battery.webp?url";
	import { user } from "$lib/auth";
	import { t } from "svelte-i18n";
</script>

<section class="features" id="features">
	<h2>{$t("pages.landing.features.header")}</h2>
	<p>
		{$t("pages.landing.features.info")}
	</p>
	<div class="feature-grid">
		<Feature
			image={weirdSmiley}
			title="{$t("pages.home.features.emote_slots")}"
			description="{$t("pages.home.features.emote_slots_info")}"
			link="{$t("pages.home.features.emote_slots_button")}"
			href="/emotes"
		/>
		<Feature
			image={cube}
			title="{$t("pages.home.features.emote_sets")}"
			description="{$t("pages.home.features.emote_sets_info")}"
			link="{$t("pages.home.features.emote_sets_button")}"
			href={$user ? `/users/${$user.id}/emote-sets` : "/login"}
		/>
		<Feature
			image={hand}
			title="{$t("pages.home.features.custom_emotes")}"
			description="{$t("pages.home.features.custom_emotes_info")}"
			link="{$t("pages.home.features.custom_emotes_button")}"
			href="/emotes"
		/>
		<Feature
			image={hourglass}
			title="{$t("pages.home.features.real-time")}"
			description="{$t("pages.home.features.real-time_info")}"
			link="{$t("pages.home.features.real-time_button")}"
			href="#download"
		/>
		<Feature
			image={megaphone}
			title="{$t("pages.home.features.large_community")}"
			description="{$t("pages.home.features.large_community_info")}"
			link="{$t("pages.home.features.large_community_button")}"
			href={PUBLIC_DISCORD_LINK}
		/>
		<Feature
			image={battery}
			title="{$t("pages.home.features.next_gen")}"
			description="{$t("pages.home.features.next_gen_info")}"
			link="{$t("pages.home.features.next_gen_button")}"
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
