import type { LayoutLoadEvent } from "./$types";

export async function load({ params }: LayoutLoadEvent) {
	// TODO: fetch emote data and return error when not found

	return {
		id: params.id,
		name: "emoteName",
		tags: ["lorem", "ipsum", "dolor", "sit", "amet"],
		author: "ayyybubu",
		channels: "2,137",
		artists: [
			{
				login: "forsen",
				displayName: "forsen",
				avatar:
					"https://static-cdn.jtvnw.net/jtv_user_pictures/forsen-profile_image-48b43e1e4f54b5c8-600x600.png",
			},
			{
				login: "nymn",
				displayName: "NymN",
				avatar: "https://cdn.7tv.app/pp/60ae3c29b2ecb015051f8f9a/71f269555aeb44c29100cae8aa59b56b",
			},
			{
				login: "troykomodo",
				displayName: "TroyKomodo",
				avatar:
					"https://static-cdn.jtvnw.net/jtv_user_pictures/3773bfdd-110b-4911-b914-6f04362a1331-profile_image-600x600.png",
			},
		],
		activity: [
			{
				kind: "reject",
				time: "1 hour ago",
				message: [
					{ text: "forsen", href: "/user/forsen", bold: true },
					{ text: "rejected personal use for" },
					{ text: "AlienDance", bold: true },
				],
			},
			{
				kind: "modify",
				time: "1 hour ago",
				message: [
					{ text: "ayyybubu", href: "/user/ayyybubu", bold: true },
					{ text: "renamed" },
					{ text: "AlienPls", bold: true },
					{ text: "to" },
					{ text: "AlienDance", bold: true },
				],
			},
			{
				kind: "approve",
				time: "1 hour ago",
				message: [
					{ text: "forsen", href: "/user/forsen", bold: true },
					{ text: "approved" },
					{ text: "AlienPls", bold: true },
					{ text: "for public listing" },
				],
			},
			{
				kind: "create",
				time: "1 hour ago",
				message: [
					{ text: "ayyybubu", href: "/user/ayyybubu", bold: true },
					{ text: "created" },
					{ text: "AlienPls", bold: true },
				],
			},
		],
	};
}
