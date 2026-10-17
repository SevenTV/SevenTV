// if a subroute doesn't have an explicity defined embed.ts,
// it will inherit the embed from it's parent up untill the root /

export enum Crawler {
	Discord = "Discordbot",
	Facebook = "facebookexternalhit",
	Twitter = "Twitterbot",
	LinkedIn = "LinkedInBot",
	Telegram = "TelegramBot",
	WhatsApp = "WhatsApp",
}

export interface EmbedContext {
	params: Record<string, string>;
	crawler: Crawler | null;
	canonicalUrl: string;
}

export type EmbedHandler = (ctx: EmbedContext) => Response | Promise<Response>;

const modules = import.meta.glob<{ default: EmbedHandler }>("/src/routes/**/embed.ts", {
	eager: true,
});

function buildMatcher(routePath: string): { regex: RegExp | null; paramNames: string[] } {
	if (routePath === "") {
		return { regex: null, paramNames: [] };
	}

	const paramNames: string[] = [];
	const pattern = routePath
		.split("/")
		.filter(Boolean)
		.map((segment) => {
			if (segment.startsWith("[") && segment.endsWith("]")) {
				paramNames.push(segment.slice(1, -1));
				return "([^/]+)";
			}
			return segment;
		})
		.join("/");

	return { regex: new RegExp(`^/${pattern}$`), paramNames };
}

function detectCrawler(url: URL): Crawler | null {
	const userAgent = url.searchParams.get("crawler");
	if (!userAgent) return null;

	for (const crawler of Object.values(Crawler)) {
		if (userAgent.includes(crawler)) {
			return crawler;
		}
	}

	return null;
}

export function resolveEmbed(
	pathname: string,
): { handler: EmbedHandler; params: Record<string, string> } | null {
	let best: { handler: EmbedHandler; params: Record<string, string>; depth: number } | null = null;
	const segments = pathname.split("/").filter(Boolean);

	for (const [file, module] of Object.entries(modules)) {
		const routePath = file
			.replace("/src/routes/", "")
			.replace(/embed\.ts$/, "")
			.replace(/\/$/, "");
		const { regex, paramNames } = buildMatcher(routePath);
		const depth = routePath.split("/").filter(Boolean).length;

		// root path "default" embed
		if (regex === null) {
			if (!best) {
				best = { handler: module.default, params: {}, depth: 0 };
			}
			continue;
		}

		for (let i = segments.length; i >= 1; i--) {
			const prefix = "/" + segments.slice(0, i).join("/");
			const match = prefix.match(regex);
			if (!match) continue;

			const params: Record<string, string> = {};
			paramNames.forEach((name, index) => (params[name] = match[index + 1]));

			if (!best || depth > best.depth) {
				best = { handler: module.default, params, depth };
			}
			break;
		}
	}

	return best;
}

export function getCanonicalUrl(url: URL): URL {
	const canonical = new URL(url.href);
	canonical.pathname = url.pathname.replace(/^\/embed/, "") || "/";
	canonical.searchParams.delete("crawler");
	return canonical;
}

export async function renderEmbed(
	handler: EmbedHandler,
	ctx: { params: Record<string, string>; url: URL },
): Promise<Response> {
	try {
		const canonicalUrl = getCanonicalUrl(ctx.url).href;
		const crawler = detectCrawler(ctx.url);

		return await handler({ params: ctx.params, crawler, canonicalUrl });
	} catch (err) {
		console.error("Embed handler error:", err);
		return new Response(null, { status: 500 });
	}
}
