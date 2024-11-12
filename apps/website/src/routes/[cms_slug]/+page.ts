import type { PageLoadEvent } from "./$types";

export function load({ params }: PageLoadEvent) {
	return {
		slug: params.cms_slug,
	};
}
