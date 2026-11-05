import type { Emote, EmoteSet } from "$/gql/graphql";

/**
 * Count how many of the given emote ids are already present in the set.
 * Emote-set items expose the underlying emote id as `item.emote?.id` (falling
 * back to `item.id`), matching the existing checks in the pickers.
 */
export function alreadyInCount(set: EmoteSet, emoteIds: string[]): number {
	const present = new Set(set.emotes.items.map((e) => e.emote?.id ?? e.id));
	return emoteIds.reduce((n, id) => (present.has(id) ? n + 1 : n), 0);
}

/**
 * The subset of `emotes` that can actually be ADDED to the set in the batch
 * (add-only) flow: not already present (by id) and whose default name does not
 * collide with a DIFFERENT emote already in the set. Already-present emotes and
 * name conflicts are skipped silently so Confirm never sends a doomed request.
 */
export function addableEmotes<T extends Pick<Emote, "id" | "defaultName">>(
	set: EmoteSet,
	emotes: T[],
): T[] {
	const presentIds = new Set(set.emotes.items.map((e) => e.emote?.id ?? e.id));
	const aliasOwner = new Map<string, string>();
	for (const item of set.emotes.items) {
		aliasOwner.set(item.alias, item.emote?.id ?? item.id);
	}
	return emotes.filter((e) => {
		if (presentIds.has(e.id)) return false; // already in the set
		const owner = aliasOwner.get(e.defaultName);
		return owner === undefined || owner === e.id; // skip name conflicts
	});
}

/** How many of `emotes` would actually be added to the set (see addableEmotes). */
export function addableCount(set: EmoteSet, emotes: Pick<Emote, "id" | "defaultName">[]): number {
	return addableEmotes(set, emotes).length;
}

/**
 * Slots the set is short for adding all of its addable emotes.
 * 0 when nothing is addable or the set has unlimited/sufficient capacity.
 */
export function addableDeficit(set: EmoteSet, emotes: Pick<Emote, "id" | "defaultName">[]): number {
	const k = addableCount(set, emotes);
	if (k === 0 || !set.capacity) return 0;
	return Math.max(0, k - (set.capacity - set.emotes.totalCount));
}

/**
 * How many extra slots the set needs to add the given emotes.
 * Returns 0 for unlimited-capacity sets (falsy `capacity`) or when every
 * not-yet-present emote already fits in the free slots.
 */
export function setDeficit(set: EmoteSet, emoteIds: string[]): number {
	if (!set.capacity) return 0;
	const newCount = emoteIds.length - alreadyInCount(set, emoteIds);
	const free = set.capacity - set.emotes.totalCount;
	return Math.max(0, newCount - free);
}
