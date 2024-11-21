export default function mouseTrap(el: HTMLElement, callback: (e: MouseEvent) => void) {
	const state = {
		el,
		willTrigger: false,
		ready: false,
	};

	setTimeout(() => {
		state.ready = true;
	}, 10);

	const onMouseDown = (e: MouseEvent) => {
		state.willTrigger = state.ready && !state.el.contains(e.target as Node);
	};

	window.addEventListener("mousedown", onMouseDown);

	const onMouseUp = (e: MouseEvent) => {
		state.willTrigger = state.willTrigger && !state.el.contains(e.target as Node);
		if (state.willTrigger) {
			callback(e);
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
