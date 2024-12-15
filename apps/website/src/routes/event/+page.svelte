<script>
	import Button from "$/components/input/button.svelte";
	import { ArrowRight } from "phosphor-svelte";

</script>
<svelte:head>
	<title>7TV Winter Games & Gift Sub Event</title>
</svelte:head>

<div class="layout">
	<div class="heading">
		<h1>7TV Winter Games & Gift Sub Event</h1>
		<Button href="https://event.7tv.app/signup" primary>
			Sign Up
			{#snippet iconRight()}
				<ArrowRight />
			{/snippet}
		</Button>
	</div>

	<p>
		Celebrate the season with 7TV's Minecraft Hunger Games Tournament and special sub-gifting event!
		Compete, gift, and enjoy exclusive rewards while engaging with your favorite streamers and their
		communities!
	</p>

	<hr />

	<h2>Minecraft Hunger Games</h2>

	<p>
		Survive and eliminate other players while staying within the shrinking playable area. Last team
		standing will be crowned as the event winner. On December 14th or 21st, the tournament will be
		streamed on the 7TV Twitch channel: <a href="https://www.twitch.tv/7tvapp">twitch.tv/7tvapp</a>
	</p>

	<h2>Tournament structure</h2>

	<h3>Community stage</h3>

	<p>
		A game of hunger games will be played between members of a streamer's community alongside the
		streamer. The winner of each game will team up with their streamer for the final stage.
	</p>

	<h3>Finals</h3>

	<p>
		Unlike the community stage, the game will be played in teams of two - the streamer and the winner
		of the community stage. Those teams will compete against other winner/streamer duos. The last team
		standing will be declared the winner.
	</p>

	<h3>Rewards</h3>

	<p>
		Winners and participating content creators participating, and those who restream the event on
		their channels will receive free 7TV subscriptions and a unique animated Winter paint!
	</p>

	<h4>For Winners</h4>

	<ul>
		<li>
			The top 3 players of the community stage will receive an exclusive Minecraft themed animated
			paint, the winner will win a free 1 year 7TV sub!
		</li>
		<li>
			The winning streamer will get a chance to create a new animated paint and let their community
			help with the design! Not great at arts? Don't worry, we can help you create one with our team
			of talented paint creators.
		</li>
	</ul>

	<h4>For Restreamers</h4>

	<ul>
		<li>
			Content creators who restream will be given free sub codes and paint codes to distribute to
			their chat during the event! (NOTE: Amount of codes given will be based on viewership size)
		</li>
	</ul>

	<h4>For Viewers</h4>

	<ul>
		<li>
			Viewers have the chance to win free subs and the exclusive animated Winter paint by watching
			streamers who restream or participate in the event!
		</li>
	</ul>

	<h4>For Participants</h4>

	<ul>
		<li>
			There will be a secret condition to unlock the exclusive animated minion paint, so don't worry
			if you don't win!
		</li>
	</ul>

	<h4>For Gifters</h4>

	<ul>
		<li>
			Everyone who gifts a 7TV subscription between 14th and 30th of December will receive a special
			animated badge!
		</li>
		<li>If you gift 10 subs during the event, you'll receive an even cooler animated badge!</li>
	</ul>

	<hr />

	<h3>How to participate</h3>

	<h4>Streamers</h4>

	Streamers interested in participating should message <a href="https://x.com/celladoor_">@celladoor_ on X</a>.
	Note: depending on the interest, we won't be able to accept everyone this time.

	<h4>Viewers</h4>

	Streamers will be allowed to pick participants of their choice (for example through raffles or
	sign-ups).

	<hr />

	<h2>Sub Gifting Event</h2>

	<h3>How It Works</h3>

	Gift 7TV subs through our redemption link <a href="/store/redeem">7tv.app/store/redeem</a> to unlock <b>a unique animated badge</b>.

	<hr />

	<h2>FAQs</h2>

	<h4>When will the tournament take place?</h4>

	<p>The tournament will be played on 14th of December OR 21st December</p>

	<h4>How can I sign up as a streamer?</h4>

	<p>Message <a href="https://x.com/celladoor_">@celladoor_ on X</a></p>

	<h4>Do I need Minecraft to participate?</h4>

	<p>
		Yes, the tournament takes place on a 7TV Minecraft server. A guide on how to join will be provided
		before the event.
	</p>

	<h4>What Minecraft version will be used?</h4>

	<p>The games will be played on Minecraft 1.21.3; however, we will use plugins that emulate PvP version from Minecraft 1.8</p>

	<h4>How can I receive the event badge?</h4>

	<p>
		Everyone who gifts a 7TV subscription between 14th and 30th of December will receive a special
		badge
	</p>

	<h4>Anything else I should know?</h4>

	<p>
		The event will be streamed on 7TV channel, and we might ask participants to be on voice channel
		during the tournament. Any malicious behavior (such as cheating during the tournament) will result
		in a permanent ban from 7TV. Rewards will be distributed via Discord or X.
	</p>
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

	h3 {
		margin: 1rem 0;
		margin-bottom: 0.6rem;
	}

	h4 {
		margin: 0.8rem 0;
		margin-bottom: 0.4rem;
	}
</style>
