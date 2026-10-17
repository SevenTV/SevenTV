import { PUBLIC_GQL_API_V4 } from "$env/static/public";
import { Crawler, type EmbedHandler } from "$lib/embedResolver";

const CARD_TYPE_LARGE = "summary_large_image";
const CARD_TYPE_SMALL = "summary";
const WIDE_ASPECT_THRESHOLD = 1.2;

interface EmoteQueryResponse {
	data: {
		emotes: {
			emote: {
				defaultName: string;
				owner: {
					mainConnection: {
						platformDisplayName: string;
					};
				};
				flags: {
					animated: boolean;
				};
				images: Array<{
					url: string;
					width: number;
					height: number;
					scale: number;
				}>;
			} | null;
		};
	} | null;
}

const embed: EmbedHandler = async ({ params, crawler, canonicalUrl }) => {
	const emoteID = params.id;

	const response = await fetch(PUBLIC_GQL_API_V4, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({
			query: `
				query OneEmote($id: Id!) {
					emotes {
						emote(id: $id) {
							defaultName
							owner {
								mainConnection {
									platformDisplayName
								}
							}
							flags {
								animated
							}
							images {
								url
								width
								height
								scale
							}
						}
					}
				}
			`,
			variables: { id: emoteID },
		}),
	});

	const result: EmoteQueryResponse = await response.json();
	const emote = result.data?.emotes?.emote;
	if (!emote) return new Response(null, { status: 404 });

	const isAnimated = emote.flags.animated;

	const pickHighestScaleImage = (predicate: (url: string) => boolean) =>
		[...emote.images]
			.filter((image) => predicate(image.url))
			.sort((a, b) => b.scale - a.scale)[0];

	// png chosen for compatibility, eg. webp didn't seem to work on twitter
	let embedImage;
	if (isAnimated) {
		embedImage = pickHighestScaleImage((url) => url.endsWith("_static.png"));
	} else {
		embedImage = pickHighestScaleImage((url) => url.endsWith("x.png"));
	}

	const title = "7TV - " + emote.defaultName;
	const emoteCreator = emote.owner.mainConnection.platformDisplayName;
	const description = "Emote by " + emoteCreator;
	const altText = `A 7TV emote titled &quot;${emote.defaultName}&quot; created by ${emoteCreator}`;
	const isWideEmote = embedImage.width / embedImage.height > WIDE_ASPECT_THRESHOLD;
	let cardType = isWideEmote ? CARD_TYPE_LARGE : CARD_TYPE_SMALL;

	switch(crawler){
		case Crawler.Discord:
			// discord only platform that supports gifs
			if (isAnimated) {
				embedImage = pickHighestScaleImage((url) => url.endsWith("x.gif"));
			}
			// for aesthetics and because it lets you save gifs to favourites
			cardType = CARD_TYPE_LARGE;
	}

	return new Response(`<!doctype html>
<html lang="en">
<head>
	<meta charset="utf-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1" />
	<link rel="icon" type="image/svg" href="/favicon.svg">
	<link rel="canonical" href="${canonicalUrl}">
	<meta name="description" content="${description}" />
	<title>${title}</title>

	<meta property="og:site_name" content="7tv.app">
	<meta property="og:url" content="${canonicalUrl}">
	<meta property="og:title" content="${title}" />
	<meta property="og:description" content="${description}" />
	<meta property="og:image" content="${embedImage.url}" />
	<meta property="og:image:width" content="${embedImage.width}" />
	<meta property="og:image:height" content="${embedImage.height}" />
	<meta property="og:type" content="website" />
	<meta property="og:image:alt" content="${altText}" />

	<meta name="twitter:site" content="https://7tv.app/" />
	<meta name="twitter:url" content="${canonicalUrl}" />
	<meta name="twitter:card" content="${cardType}" />
	<meta name="twitter:title" content="${title}" />
	<meta name="twitter:description" content="${description}" />
	<meta name="twitter:image" content="${embedImage.url}" />
	<meta name="theme-color" content="#aa71ff">
</head>
<body></body>
</html>`, {
		headers: { "Content-Type": "text/html; charset=utf-8" },
	});
};

export default embed;
