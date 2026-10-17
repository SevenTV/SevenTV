// example "default" 7tv embed, design not finalized

import type { EmbedHandler } from "$lib/embedResolver";

const embed: EmbedHandler = () => {
	return new Response(
		`<!doctype html>
<html lang="en">
<head>
	<meta charset="utf-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1" />
	<title>7TV</title>
	<meta property="og:title" content="7TV" />
	<meta property="og:description" content="The Emote Platform for All" />
	<meta property="og:type" content="website" />
	<meta property="og:image" content="https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQDl8gQDBIlc3U-QfB3qHKUUqq1wzGAdLeHgKBHhv6KpkW4W726HMQOppMK&s=10.png" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="7TV" />
	<meta name="twitter:image" content="https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQDl8gQDBIlc3U-QfB3qHKUUqq1wzGAdLeHgKBHhv6KpkW4W726HMQOppMK&s=10.png" />
	<meta name="twitter:description" content="The Emote Platform for All" />
	<meta name="theme-color" content="#aa71ff">
</head>
<body></body>
</html>`,
		{ headers: { "Content-Type": "text/html; charset=utf-8" } },
	);
};

export default embed;
