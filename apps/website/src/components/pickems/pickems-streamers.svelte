<script lang="ts">
	import { ArrowUpRight } from "phosphor-svelte";

	// TODO update names an add profile pictures
	let streamers = [
		"xQc",
		"OHNE",
		"JASON",
		"EROBB",
		"ARROW",
		"FURI",
		"OMIE",
		"DONA",
		"DIMA",
		"DORO",
		"DIZZY",
		"DUWAP",
		"RENYAN",
		"STEWIE",
		"TRILLUXE",
	];
</script>

<div class="streamer-grid">
	{#each streamers as streamer}
		<div class="streamer">
			<img
				class="streamer-pfp"
				alt={`${streamer}'s profile picture`}
				src="https://cdn.7tv.app/user/01FF3DRSGG000FF5VVCKV49EA1/profile-picture/01JJCJQ9HDR7HH3PH3APTW9SS7/3x.webp"
			/>
			<div class="info">
				<h3>
					{streamer}
				</h3>
				<a href={`https://twitch.tv/${streamer}`} target="_blank">
					<u> VIEW TWITCH </u>
					<ArrowUpRight />
				</a>
			</div>
		</div>
	{/each}
</div>

<style lang="scss">
	.streamer-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));

		.streamer {
			display: flex;
			padding: 1rem;
			gap: 1rem;
			vertical-align: middle;
			justify-content: center;

			img {
				border-radius: 1rem;
				height: 4rem;
				width: 4rem;
			}

			.info {
				padding-top: 1rem;
				margin-block: auto;
				vertical-align: baseline;

				a {
					display: flex;
					vertical-align: middle;
					color: grey;

					:global(svg) {
						vertical-align: baseline;
					}
				}
			}
		}
	}
</style>
