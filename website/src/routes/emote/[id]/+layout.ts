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
				type: "rejected-personal",
				author: "forsen",
				emoteName: "emoteName",
				time: "1 hour hago",
			},
			{
				type: "renamed",
				author: "ayyybubu",
				oldName: "AlienPls3",
				emoteName: "emoteName",
				time: "1 hour hago",
			},
			{
				type: "approved",
				author: "forsen",
				emoteName: "emoteName",
				time: "1 hour hago",
			},
			{
				type: "created",
				author: "ayyybubu",
				emoteName: "emoteName",
				time: "1 hour hago",
			},
		],
	};
}
