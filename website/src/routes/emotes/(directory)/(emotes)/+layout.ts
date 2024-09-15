import type { LayoutLoadEvent } from "./$types";

export function load({ url }: LayoutLoadEvent) {
    const query = url.searchParams.get("q");

    const pageQuery = url.searchParams.get("p");
    const page = pageQuery ? Number(pageQuery) : null;

    return {
        query,
        page,
    };
}
