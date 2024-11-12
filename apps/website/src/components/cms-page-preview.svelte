<script lang="ts">
	import moment from "moment/min/moment-with-locales";
	import Button from "./input/button.svelte";
	import { DotsThree } from "phosphor-svelte";
	import DropDown from "./drop-down.svelte";
	import FromNow from "./from-now.svelte";

	let { slug }: { slug: string } = $props();
</script>

<a class="preview" href="/{slug}">
	<img
		class="profile-picture"
		src="/test-profile-pic.jpeg"
		width={2.5 * 16}
		height={2.5 * 16}
		alt="profile"
	/>
	<div class="content">
		<DropDown style="position: absolute; top: 0; right: 0;">
			<Button>
				{#snippet icon()}
					<DotsThree />
				{/snippet}
			</Button>
		</DropDown>
		<span class="info">
			<span class="author">ayyybubu</span>
			<span class="date"> · <FromNow date={moment("2024-03-12T12:00:00")} /></span>
		</span>
		<article class="text">
			<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>
			<p>
				Nam ante turpis, vehicula vel ligula eu, imperdiet fringilla augue. Vestibulum ornare
				molestie gravida. Integer non augue pharetra enim maximus convallis in sit amet quam.
			</p>
			<p>
				Ut mollis consectetur augue, sit amet auctor risus malesuada id. Maecenas sit amet mi a mi
				consectetur fringilla. Praesent pretium ut mi sed interdum.
			</p>
		</article>
	</div>
</a>

<style lang="scss">
	.preview {
		padding: 1rem;

		display: flex;
		gap: 0.75rem;
		text-decoration: none;
		color: var(--text);

		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		transition: background-color 0.1s;
		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}
	}

	.profile-picture {
		border-radius: 50%;
		border: 2px solid var(--staff);
	}

	.content {
		flex-grow: 1;

		position: relative;
	}

	.info {
		font-size: 0.875rem;

		.author {
			color: var(--staff);
			font-weight: 600;
		}

		.date {
			color: var(--text-light);
		}
	}

	.text {
		font-size: 0.875rem;

		& > p {
			margin-top: 0.5rem;
		}
	}
</style>
