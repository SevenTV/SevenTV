<script lang="ts">
	import { user } from "$/lib/stores";
	import Role from "../profile/role.svelte";
	import { fade } from "svelte/transition";
	import { CaretRight, ChartLine, ChatDots, Code, Compass, Gear, GlobeHemisphereWest, House, IconContext, LockSimple, Moon, Note, PaintBrush, Question, SealCheck, SignOut, Smiley, Star } from "phosphor-svelte";
</script>

<IconContext values={{ size: "1.5rem" }}>
	<div class="menu" transition:fade={{ duration: 100 }}>
		{#if $user}
			<a class="profile" href="/user/ayyybubu">
				<img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
				<span class="name">
					ayyybubu
					<SealCheck size="0.8rem" />
				</span>
				<div class="roles">
					<Role name="Staff" />
					<Role name="Subscriber" />
				</div>
				<div class="chevron">
					<CaretRight size="1.2rem" />
				</div>
			</a>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list hide-on-desktop">
			<a href="/">
				<House />
				Home
			</a>
			<a href="/emotes">
				<Smiley />
				Emotes
			</a>
			<a href="/discover">
				<Compass />
				Discover
			</a>
			<a href="/store" class="store">
				<Star />
				Store
			</a>
		</div>
		{#if $user}
			<div class="link-list">
				<a href="/cosmetics">
					<PaintBrush />
					Cosmetics
				</a>
				<a href="/analytics">
					<ChartLine />
					Analytics
				</a>
			</div>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list">
			<button>
				<GlobeHemisphereWest />
				Language
				<div class="chevron">
					<CaretRight />
				</div>
			</button>
			<button>
				<Moon />
				Theme
				<div class="chevron">
					<CaretRight />
				</div>
			</button>
			{#if $user}
				<a href="/settings" class="hide-on-mobile">
					<Gear />
					Settings
				</a>
				<button class="hide-on-desktop">
					<Gear />
					Settings
					<div class="chevron">
						<CaretRight />
					</div>
				</button>
			{/if}
		</div>
		<hr class="hide-on-mobile" />
		<div class="link-list">
			<a href="/developer">
				<Code />
				Developer Portal
			</a>
			<a href="/contact">
				<ChatDots />
				Contact
			</a>
			<a href="/faq">
				<Question />
				FAQ
			</a>
			<a href="/privacy">
				<LockSimple />
				Privacy Policy
			</a>
			<a href="/tos">
				<Note />
				Terms of Service
			</a>
		</div>
		{#if $user}
			<hr class="hide-on-mobile" />
			<div class="link-list">
				<button on:click={() => ($user = false)}>
					<SignOut />
					Sign out
				</button>
			</div>
		{/if}
	</div>
</IconContext>

<style lang="scss">
	.menu {
		display: flex;
		flex-direction: column;

		text-align: left;

		min-width: 16rem;
	}

	.profile {
		color: var(--text);
		text-decoration: none;
		padding: 1rem;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		display: grid;
		grid-template-columns: auto auto 1fr;
		grid-template-rows: auto auto;
		align-items: center;
		row-gap: 0.5rem;
		column-gap: 0.75rem;

		.profile-picture {
			grid-row: 1 / -1;

			width: 3rem;
			height: 3rem;
			border-radius: 50%;
			border: 2px solid var(--staff);
		}

		.name {
			grid-row: 1;
			font-size: 1rem;
			font-weight: 600;
			color: var(--staff);
		}

		.roles {
			grid-row: 2;

			display: flex;
			gap: 0.25rem;
		}

		.chevron {
			grid-row: 1 / -1;
			justify-self: end;

			color: var(--text);
		}

		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}
	}

	.link-list {
		display: flex;
		flex-direction: column;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		a,
		button {
			padding: 0.75rem;
			border-radius: 0.5rem;
			color: var(--text);
			font-size: 0.875rem;
			font-weight: 500;
			text-decoration: none;

			display: flex;
			align-items: center;
			gap: 1rem;

			&:hover,
			&:focus-visible {
				background-color: var(--bg-light);
			}

			.chevron {
				flex-grow: 1;
				justify-self: end;
				text-align: right;
			}
		}

		.store {
			color: var(--subscriber);
		}
	}

	@media screen and (max-width: 960px) {
		.menu {
			padding: 0.5rem 1rem;
			gap: 0.5rem;
		}

		.link-list {
			a,
			button {
				padding: 1rem;
				font-size: 1rem;
				gap: 0.75rem;
			}
		}
	}
</style>
