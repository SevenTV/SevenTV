import type { LayoutLoadEvent } from "./$types";

export function load({ params }: LayoutLoadEvent) {
	return {
		id: params.id,
	};
}
