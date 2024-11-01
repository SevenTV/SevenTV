import type { ActionReturn } from "svelte/action";

interface Attributes {
	"on:outsideClick": (e: CustomEvent<MouseEvent>) => void;
}

export default function mouseTrap(el: HTMLElement): ActionReturn<undefined, Attributes> {
	const state = {
		el,
		willTrigger: false,
		ready: false,
	};

	setTimeout(() => {
		state.ready = true;
	}, 10);

	const onMouseDown = (e: MouseEvent) => {
		state.willTrigger = state.ready && e.button === 0 && !state.el.contains(e.target as Node);
	};

	window.addEventListener("mousedown", onMouseDown);

	const onMouseUp = (e: MouseEvent) => {
		state.willTrigger = state.willTrigger && e.button === 0 && !state.el.contains(e.target as Node);
		if (state.willTrigger) {
			el.dispatchEvent(new CustomEvent("outsideClick", { detail: e }));
		}
	};

	window.addEventListener("mouseup", onMouseUp);

	return {
		destroy: () => {
			window.removeEventListener("mousedown", onMouseDown);
			window.removeEventListener("mouseup", onMouseUp);
		},
	};
}
