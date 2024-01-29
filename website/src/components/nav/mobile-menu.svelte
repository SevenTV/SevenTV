<script lang="ts">
	import {
		faChevronRight,
		faCode,
		faBadgeCheck,
		faEarthAmericas,
	} from "@fortawesome/pro-solid-svg-icons";
	import {
		faCompass,
		faMessageArrowUpRight,
		faMoon,
		faQuestionCircle,
		faSmileWink,
		faStar,
		faHome,
		faGear,
		faLock,
		faMemo,
		faRightFromBracket,
	} from "@fortawesome/pro-regular-svg-icons";
	import Fa from "svelte-fa";
	import { user } from "$/lib/stores";
	import Role from "../profile/role.svelte";
</script>

<div class="menu">
	<a class="profile" href="/user/ayyybubu">
		<img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
		<span class="name">
			ayyybubu
			<Fa icon={faBadgeCheck} size="0.75x" />
		</span>
		<div class="roles">
			<Role name="Staff" />
			<Role name="Subscriber" />
		</div>
		<div class="chevron">
			<Fa icon={faChevronRight} />
		</div>
	</a>
	<div class="link-list">
		<a href="/">
			<Fa icon={faHome} size="1.2x" fw />
			Home
		</a>
		<a href="/emotes">
			<Fa icon={faSmileWink} size="1.2x" fw />
			Emotes
		</a>
		<a href="/discover">
			<Fa icon={faCompass} size="1.2x" fw />
			Discover
		</a>
		<a href="/store">
			<Fa icon={faStar} size="1.2x" fw />
			Store
		</a>
	</div>
	<div class="link-list">
		<button>
			<Fa icon={faEarthAmericas} size="1.2x" fw />
			Language
			<div class="chevron">
				<Fa icon={faChevronRight} />
			</div>
		</button>
		<button>
			<Fa icon={faMoon} size="1.2x" fw />
			Theme
			<div class="chevron">
				<Fa icon={faChevronRight} />
			</div>
		</button>
		<button>
			<Fa icon={faGear} size="1.2x" fw />
			Settings
			<div class="chevron">
				<Fa icon={faChevronRight} />
			</div>
		</button>
	</div>
	<div class="link-list">
		<a href="/developer">
			<Fa icon={faCode} size="1.2x" fw />
			Developer Portal
		</a>
		<a href="/contact">
			<Fa icon={faMessageArrowUpRight} size="1.2x" fw />
			Contact
		</a>
		<a href="/faq">
			<Fa icon={faQuestionCircle} size="1.2x" fw />
			FAQ
		</a>
		<a href="/privacy">
			<Fa icon={faLock} size="1.2x" fw />
			Privacy Policy
		</a>
		<a href="/tos">
			<Fa icon={faMemo} size="1.2x" fw />
			Terms of Service
		</a>
	</div>
	<div class="link-list">
		<button on:click={() => ($user = false)}>
			<Fa icon={faRightFromBracket} size="1.2x" fw />
			Sign out
		</button>
	</div>
</div>

<style lang="scss">
	.menu {
		padding: 0.5rem 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;
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
			padding: 0 1rem;
			height: 3.5rem;
			border-radius: 0.5rem;
			color: var(--text);
			font-size: 1rem;
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
	}
</style>
