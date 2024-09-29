import type { Filters } from "$/gql/graphql";
import { TAGS_SEPERATOR } from "../+layout.svelte";
import type { LayoutLoadEvent } from "./$types";

export function load({ url }: LayoutLoadEvent) {
    const query = url.searchParams.get("q");

	const tags = url.searchParams.get("t");
	const tagsArray = tags ? tags.split(TAGS_SEPERATOR) : [];

    const pageQuery = url.searchParams.get("p");
    const page = pageQuery ? Number(pageQuery) : null;

	let filters: Filters = {};

	if (url.searchParams.get("a") === "1") {
		filters.animated = true;
	}

	if (url.searchParams.get("s") === "1") {
		filters.animated = false;
	}

	if (url.searchParams.get("o") === "1") {
		filters.overlaying = true;
	}

	if (url.searchParams.get("e") === "1") {
		filters.exactMatch = true;
	}

    return {
        query,
		tags: tagsArray,
        page,
		filters,
    };
}
