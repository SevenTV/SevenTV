<script lang="ts">
	import { CaretDown, DeviceMobile } from "phosphor-svelte";
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
	import ChatterinoLogo from "../icons/chatterino-logo.svelte";
	import DropDown from "../drop-down.svelte";
	import MenuButton from "../input/menu-button.svelte";
	import {
		PUBLIC_CHATSEN_LINK,
		PUBLIC_FROSTY_LINK,
		PUBLIC_DANK_CHAT_LINK,
	} from "$env/static/public";
</script>

<section class="download" id="download">
	<div class="weird-smiley hide-on-mobile" aria-hidden="true">
		<img src="/weird-smiley.png" alt="Weird Smiley" />
	</div>
	<h2>
		Download
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
		Start your journey with hundreds of emotes and features that enhance your chatting experience.
		Download the Official 7tv Extension or other 3rd party tools.
	</p>
	<div class="buttons">
		<DropDown>
			<Button primary style="font-size: 1em;">
				{#snippet icon()}
					<Logo size={1.25 * 16} />
				{/snippet}
				Web Extension
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			{#snippet dropdown()}
				<div>Idk why this is a dropdown</div>
			{/snippet}
		</DropDown>
		<DropDown>
			<Button secondary style="font-size: 1em;">
				{#snippet icon()}
					<ChatterinoLogo />
				{/snippet}
				Chatterino
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			{#snippet dropdown()}
				<div>Idk why this is a dropdown</div>
			{/snippet}
		</DropDown>
		<DropDown align="left">
			<Button secondary style="font-size: 1em;">
				{#snippet icon()}
					<DeviceMobile />
				{/snippet}
				Mobile Apps
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			{#snippet dropdown()}
				<div class="apps-dropdown">
					<MenuButton href={PUBLIC_CHATSEN_LINK}>Chatsen</MenuButton>
					<MenuButton href={PUBLIC_FROSTY_LINK}>Frosty</MenuButton>
					<MenuButton href={PUBLIC_DANK_CHAT_LINK}>DankChat</MenuButton>
				</div>
			{/snippet}
		</DropDown>
	</div>
</section>

<style lang="scss">
	.download {
		text-align: center;
		max-width: 45rem;
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

		.apps-dropdown {
			min-width: 10rem;

			display: flex;
			flex-direction: column;
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
