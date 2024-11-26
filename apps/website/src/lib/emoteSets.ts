import { writable } from "svelte/store";
import { user } from "./auth";
import { subscribe } from "./eventApi";
import type { EmoteSet, EmoteSetEmote } from "$/gql/graphql";
import { gqlClient } from "./gql";
import { graphql } from "$/gql";
import { DispatchType, type DispatchPayload } from "$/workers/eventApiWorkerTypes";

export const editableEmoteSets = writable<EmoteSet[]>([]);

async function queryEmoteSets(setIds: string[]) {
	let sets: EmoteSet[] = [];

	for (let i = 0; i < setIds.length; i += 50) {
		const chunk = setIds.slice(i, i + 50);

		const res = await gqlClient()
			.query(
				graphql(`
					query EditableEmoteSets($setIds: [Id!]!) {
						emoteSets {
							emoteSets(ids: $setIds) {
								id
								name
								owner {
									id
									mainConnection {
										platformDisplayName
									}
								}
								capacity
								kind
								tags
								emotes {
									items {
										id
										alias
									}
									totalCount
								}
							}
						}
					}
				`),
				{ setIds: chunk },
			)
			.toPromise();

		if (res.data) {
			sets = sets.concat(res.data.emoteSets.emoteSets as EmoteSet[]);
		}
	}

	return sets;
}

user.subscribe((user) => {
	if (user) {
		queryEmoteSets(user.editableEmoteSetIds).then((sets) => {
			editableEmoteSets.set(sets);
		});
	}
});

editableEmoteSets.subscribe((editableEmoteSets) => {
	const unsubscribers: (() => void)[] = [];

	if (editableEmoteSets) {
		// Subscribe to event api topics
		for (const emoteSet of editableEmoteSets) {
			unsubscribers.push(
				subscribe(
					DispatchType.EmoteSetUpdate,
					emoteSet.id,
					onEmoteSetUpdate,
					`editableSets:${emoteSet.id}`,
				),
			);
		}
	}

	return () => {
		for (const unsubscribe of unsubscribers) {
			unsubscribe();
		}
	};
});

function onEmoteSetUpdate(payload: DispatchPayload) {
	editableEmoteSets.update((editableEmoteSets) => {
		return editableEmoteSets.map((emoteSet) => {
			if (emoteSet.id === payload.body.id) {
				// Added emotes
				for (const change of payload.body.pushed ?? []) {
					if (change.key === "emotes" && change.value) {
						emoteSet.emotes.items.push({
							id: change.value.id,
							alias: change.value.name,
						} as EmoteSetEmote);
						emoteSet.emotes.totalCount++;
					}
				}

				// Removed emotes
				for (const change of payload.body.pulled ?? []) {
					if (change.key === "emotes" && change.old_value) {
						const old_value = change.old_value;
						emoteSet.emotes.items = emoteSet.emotes.items.filter(
							(emote) => !(emote.id === old_value.id && emote.alias === old_value.name),
						);
						emoteSet.emotes.totalCount--;
					}
				}
			}

			return emoteSet;
		});
	});
}
