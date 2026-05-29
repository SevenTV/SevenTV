<script lang="ts">
	import { ArrowSquareOut, CaretDown, DeviceMobile, Robot } from "phosphor-svelte";
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
	import ChatterinoLogo from "../icons/chatterino-logo.svelte";
	import DropDown from "../drop-down.svelte";
	import {
		PUBLIC_CHATSEN_LINK,
		PUBLIC_FROSTY_LINK,
		PUBLIC_DANK_CHAT_LINK,
		PUBLIC_EXTENSION_CWS_NIGHTLY,
		PUBLIC_EXTENSION_CWS_RECOMMENDED,
		PUBLIC_EXTENSION_CWS_STABLE,
		PUBLIC_EXTENSION_FF_NIGHTLY,
		PUBLIC_EXTENSION_FF_RECOMMENDED,
		PUBLIC_EXTENSION_FF_STABLE,
		PUBLIC_TWICK_LINK,
	} from "$env/static/public";
	import weirdSmiley from "$assets/weird-smiley.webp?url";
	import FrostyLogo from "../icons/frosty-logo.svelte";
	import DankchatLogo from "../icons/dankchat-logo.svelte";
	import ChatsenLogo from "../icons/chatsen-logo.svelte";
	import TwickLogo from "$assets/twick-logo.png?url";
	import SevenTvNewExtensionLogo from "$assets/7tv-new-extension.png?url";
	import { browser } from "$app/environment";
	import { t } from "svelte-i18n";

	const isFirefox = browser ? navigator.userAgent.includes("Firefox") : false;

	const KNOWN_7TV_BOTS = [
		{
			name: "PotatBotat",
			pfp: "https://static-cdn.jtvnw.net/jtv_user_pictures/82d5fbab-005e-4711-89e1-dec1ab44a5e9-profile_image-300x300.png",
			website: "https://potat.app/?referrer=7tv.app",
		},
		{
			name: "Spanixbot",
			pfp: "https://static-cdn.jtvnw.net/jtv_user_pictures/7994f653-a946-45b8-9e17-3d9d9c14f386-profile_image-300x300.png",
			website: "https://bot.spanix.team/?referrer=7tv.app",
		},
		{
			name: "Susgeebot",
			pfp: "https://cdn.7tv.app/user/01FY4X0H4G00077WTWSFD40A96/profile-picture/01GWW4X5ER000CQWSSY6RX1AAR/2x.avif",
			website: "https://susgee.dev/?referrer=7tv.app",
		},
		{
			name: "Gempbot",
			pfp: "https://static-cdn.jtvnw.net/jtv_user_pictures/f549ebb7-eadd-4b29-9735-f09e5f8c2114-profile_image-300x300.png",
			website: "http://bot.gempir.com/?referrer=7tv.app",
		},
		{
			name: "Turtegbot",
			pfp: "https://static-cdn.jtvnw.net/jtv_user_pictures/d012b225-1c05-4860-9d47-dcd056bdfccc-profile_image-300x300.png",
			website: "https://turteg.xslash.ovh/?referrer=7tv.app",
		},
		{
			name: "Rvbies",
			pfp: "https://static-cdn.jtvnw.net/jtv_user_pictures/1bf6f7ff-84a4-4f06-8367-589056556bc0-profile_image-300x300.png",
			website: "https://rvbies.dev/?referrer=7tv.app",
		},
	];
</script>

<section class="download" id="download">
	<div class="weird-smiley hide-on-mobile" aria-hidden="true">
		<img src={weirdSmiley} alt="Weird Smiley" />
	</div>
	<h2>
		{$t("labels.download")}
		<svg
			aria-hidden="true"
			class="star hide-on-mobile"
			width="65"
			height="65"
			viewBox="0 0 55 55"
			fill="none"
			xmlns="http://www.w3.org/2000/svg"
		>
			<path
				d="M55 27.4804C44.2118 28.4604 38.0136 29.3621 34.2083 32.3414C29.7361 35.8304 28.6769 42.2202 27.5 55C26.2839 41.7498 25.1854 35.3991 20.2817 31.9886C16.4765 29.3229 10.3174 28.4604 0 27.5196C10.7489 26.5396 16.9864 25.6379 20.7525 22.6978C25.2639 19.1696 26.3231 12.819 27.5 0C28.5984 11.7997 29.5792 18.1112 33.2275 21.7569C36.8759 25.4027 43.2311 26.422 55 27.4804Z"
				fill="currentColor"
			/>
		</svg>
	</h2>
	<p>
		{$t("pages.landing.download.start")}
	</p>
	<div class="buttons">
		<DropDown align="left">
			<Button primary style="font-size: 1em;">
				{#snippet icon()}
					<Logo size={1.25 * 16} />
				{/snippet}
				{$t("pages.landing.download.extension")}
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			{#snippet dropdown()}
				<div class="button-list">
					<Button
						big
						secondary
						href={isFirefox ? PUBLIC_EXTENSION_FF_RECOMMENDED : PUBLIC_EXTENSION_CWS_RECOMMENDED}
					>
						{#snippet icon()}
							<img src={SevenTvNewExtensionLogo} alt="7TV Extension Logo" style="height: 1.25rem;" />
						{/snippet}
						<span>{$t("pages.landing.download.recommended")}</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
					<Button
						big
						secondary
						href={isFirefox ? PUBLIC_EXTENSION_FF_STABLE : PUBLIC_EXTENSION_CWS_STABLE}
					>
						{#snippet icon()}
							<Logo size={1.25 * 16} />
						{/snippet}
						<span>{$t("pages.landing.download.stable")}</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
					<Button
						big
						secondary
						href={isFirefox ? PUBLIC_EXTENSION_FF_NIGHTLY : PUBLIC_EXTENSION_CWS_NIGHTLY}
					>
						{#snippet icon()}
							<Logo size={1.25 * 16} style="color: var(--store)" />
						{/snippet}
						<span>{$t("pages.landing.download.nightly")}</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
				</div>
			{/snippet}
		</DropDown>
		<Button
			href="https://github.com/SevenTV/chatterino7/releases"
			secondary
			style="font-size: 1em;"
		>
			{#snippet icon()}
				<ChatterinoLogo />
			{/snippet}
			Chatterino
			{#snippet iconRight()}
				<ArrowSquareOut />
			{/snippet}
		</Button>
		<DropDown align="left">
			<Button secondary style="font-size: 1em;">
				{#snippet icon()}
					<DeviceMobile />
				{/snippet}
				<span>{$t("pages.landing.download.mobile.apps")}</span>
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			{#snippet dropdown()}
				<div class="button-list">
					<Button big secondary href={PUBLIC_CHATSEN_LINK}>
						{#snippet icon()}
							<ChatsenLogo />
						{/snippet}
						<span>Chatsen</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
					<Button big secondary href={PUBLIC_FROSTY_LINK}>
						{#snippet icon()}
							<FrostyLogo />
						{/snippet}
						<span>Frosty</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
					<Button big secondary href={PUBLIC_DANK_CHAT_LINK}>
						{#snippet icon()}
							<DankchatLogo />
						{/snippet}
						<span>DankChat</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
					<Button big secondary href={PUBLIC_TWICK_LINK}>
						{#snippet icon()}
							<img src={TwickLogo} alt="Twick Logo" style="height: 1.5rem;" />
						{/snippet}
						<span>Twick</span>
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
					</Button>
				</div>
			{/snippet}
		</DropDown>
		<DropDown align="left">
			<Button secondary style="font-size: 1em;">
				{#snippet icon()}
					<Robot />
				{/snippet}
				<span>{$t("pages.landing.download.bots.header")}</span>
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			{#snippet dropdown()}
				<div class="button-list">
					{#each KNOWN_7TV_BOTS as bot (bot.name)}
						<Button big secondary href={bot.website}>
							{#snippet icon()}
								<img
									src={bot.pfp}
									alt="{bot.name} Profile Picture"
									style="height: 1.5rem; border-radius: 50%;"
								/>
							{/snippet}
							<span>{bot.name}</span>
							{#snippet iconRight()}
								<ArrowSquareOut />
							{/snippet}
						</Button>
					{/each}
				</div>
			{/snippet}
		</DropDown>
	</div>
</section>

<style lang="scss">
	.download {
		text-align: center;
		max-width: 60rem;
		margin: 0 auto;
		font-size: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		align-items: center;

		position: relative;

		h2 {
			font-family: "AKONY";
			font-size: 2.8rem;
			font-weight: 700;

			word-break: break-word;

			.star {
				position: absolute;
				transform: translateY(-75%);
			}
		}

		p {
			font-size: 1.125rem;
			font-weight: 400;
			color: var(--text-light);
			line-height: 1.8rem;
		}

		.button-list {
			display: flex;
			flex-direction: column;

			& > :global(.button) {
				border: none;

				&:not(:hover, :focus-visible) {
					background-color: transparent;
				}
			}

			& > :global(.button > span) {
				flex-grow: 1;
			}
		}

		.weird-smiley {
			position: absolute;
			z-index: 0;
			top: -8rem;
			left: -10rem;

			img {
				width: 9rem;
			}

			&::before {
				content: "";
				position: absolute;
				z-index: -1;
				top: -20%;
				left: -20%;
				right: -20%;
				bottom: -20%;
				background: radial-gradient(
					circle at bottom,
					rgba(56, 0, 176, 0.83),
					rgba(0, 215, 244, 0.5)
				);
				filter: blur(100px);
			}
		}
	}

	.buttons {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		justify-content: center;
	}

	@media screen and (max-width: 1260px) {
		.download {
			max-width: 45rem;
		}
	}

	@media screen and (max-width: 960px) {
		.download {
			font-size: 0.875rem;

			h2 {
				font-size: 2rem;
			}

			p {
				font-size: 1rem;
			}
		}
	}
</style>
