<script lang="ts">
	import pickemsBanner from "$assets/cs3.jpg";
	import LandingNav from "$/components/landing/landingNav.svelte";
	import Button from "$/components/input/button.svelte";
	import { ArrowRight } from "phosphor-svelte";
</script>

<svelte:head>
	<title>Pickems Pass Event</title>
</svelte:head>

<div class="layout">
	<LandingNav />

	<div class="header-wrapper">
		<img src={pickemsBanner} alt="pickems banner" />
	</div>

	<div class="heading">
		<h1>About the Tournament</h1>
	</div>

	<p>
		Get exclusive access to the 7TV Pickems CS tournament. Stream and chat with the best CS gamers
		on the planet as they battle. The Tournament starts on the 28th of February!
	</p>

	<div class="pricing">
		<div class="pricing-item">
			<div class="h">Tournament Pass Only</div>
			<div class="z">
				<div class="s">
					<p>Regular Price</p>
				</div>
				<div class="o">
					<div class="b"><s>€4.99</s></div>
				</div>
			</div>

			<div class="z">
				<div class="s">
					<p>Early-Bird Discount</p>
				</div>
				<div class="o">
					<div class="b">€4.00</div>
				</div>
			</div>
			<div class="cen">
				<Button primary style="background-color: #ffa200"
					>Buy Now
					{#snippet iconRight()}
						<ArrowRight />
					{/snippet}
				</Button>
			</div>
			<div class="cnd">€4.00 today, no subscription</div>
		</div>

		<div class="pricing-item">
			<div class="h">Pass + 7TV Monthly Sub</div>
			<div class="z">
				<div class="s">
					<p>Regular Price</p>
				</div>
				<div class="o">
					<div class="b"><s>€7.99</s></div>
				</div>
			</div>

			<div class="z">
				<div class="s">
					<p>Early-Bird Discount</p>
				</div>
				<div class="o">
					<div class="b">€7.00</div>
				</div>
			</div>
			<div class="cen">
				<Button primary style="background-color: #ffa200"
					>Buy Now
					{#snippet iconRight()}
						<ArrowRight />
					{/snippet}
				</Button>
			</div>
			<div class="cnd">€7.00 today, then €3.99 a month</div>
		</div>

		<div class="pricing-item">
			<div class="h">Pass + 7TV Yearly Sub</div>
			<div class="z">
				<div class="s">
					<p>Regular Price</p>
				</div>
				<div class="o">
					<div class="b"><s>€45.99</s></div>
				</div>
			</div>

			<div class="z">
				<div class="s">
					<p>Early-Bird Discount</p>
				</div>
				<div class="o">
					<div class="b">€42.99</div>
				</div>
			</div>
			<div class="cen">
				<Button primary style="background-color: #ffa200"
					>Buy Now
					{#snippet iconRight()}
						<ArrowRight />
					{/snippet}
				</Button>
			</div>
			<div class="cnd">€42.99 today, then €39.99 a year</div>
		</div>
	</div>

	<hr />

	<h2>Tournament Overview</h2>
	<p>Playoffs: Feb 28 (Fri) – 2nd March (Sun)</p>

	<h2>Format:</h2>
	<ul>
		<li>
			16 streamers, each assembling a team of 4/5 people from their community to enter the Playoffs.
		</li>
		<li>Streamers can choose to coach or play</li>
		<li>16 Team Single Elimination Bracket</li>
		<li>Final Showmatch: a fun clash with streamers or pros</li>
	</ul>

	<h2>Pick’ems with 7TV</h2>
	<p>
		Viewers can buy a Pick’ems Pass on the 7TV site, earning a unique badge and some extra cosmetics
		if they predict the matches correctly.
	</p>

	<h2>Why It’s Worth Your Time</h2>
	<ul>
		<li>
			Two weekends packed with CS action, 15 other streamers, and tons of potential for creative
			content on your channel.
		</li>
	</ul>
	<p></p>
	<h2>Viewers</h2>
	<p>
		CS tournaments are pulling in big crowds these days, and with 7TV’s cosmetic rewards for correct
		predictions, there’s even more hype.
	</p>
	<h2>Community</h2>
	<p>
		You get to handpick your teams—maybe host tryouts, run a raffle, or anything else that fits your
		style. It’s a perfect opportunity to bring your community together.
	</p>
	<h2>Bragging Rights</h2>
	<p>If your team wins, you’ll have the ultimate flex—your community is the best at CS!</p>
</div>

<style lang="scss">
	.layout {
		max-width: 60rem;
		margin: 2rem auto;
		padding: 0 1rem;
	}

	hr {
		margin: 1rem 0;
	}

	.heading {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	h1 {
		margin: 1.4rem 0;
		margin-bottom: 1rem;
	}

	h2 {
		margin: 1.2rem 0;
		margin-bottom: 0.8rem;
	}

	<!-- h3 { -->
	<!-- 	margin: 1rem 0; -->
	<!-- 	margin-bottom: 0.6rem; -->
	<!-- } -->
	<!---->
	<!-- h4 { -->
	<!-- 	margin: 0.8rem 0; -->
	<!-- 	margin-bottom: 0.4rem; -->
	<!-- } -->

	.header-wrapper {
		height: 300px;
		background: #0a0a0a;
		padding: 0;
		overflow: hidden;
		backdrop-filter: blur(50px);
		border-radius: 1.5rem;
		border: 1px solid var(--border-active);
		background-color: rgba(121, 121, 121, 0.04);
	}

	.header-wrapper img {
		width: 100%;
		margin-top: -50px;
	}

	.pricing {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		margin-top: 2rem;
		margin-bottom: 2rem;
	}

	.pricing-item {
		width: 100%;
		padding: 1.5rem;
		background: #000000;
		border-radius: 1rem;
		border: 1px solid #ffa200;
		color: #ffa200;
		box-shadow: -2px 0px 15px rgba(255, 225, 0, 0.06);
	}

	.pricing-item .h {
		font-size: 1.3rem;
		font-weight: bold;
		margin-bottom: 1rem;
		text-align: center;
	}

	.pricing-item .z {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.pricing-item .z .b {
		color: white;
		font-size: 1.5rem;
		margin-left: auto;
	}

	.pricing-item .z .s {
		font-size: 1rem;
	}

	.pricing-item .z .b s {
		color: #888888;
	}

	.cen {
		display: flex;
		justify-content: center;
		padding-top: 1rem;
	}

	.cnd {
		display: flex;
		justify-content: center;
		font-size: 0.75rem;
		padding-top: 1rem;
		align-items: center;
		gap: 0.2rem;
	}
</style>
