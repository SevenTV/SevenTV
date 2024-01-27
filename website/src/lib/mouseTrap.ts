export default function mouseTrap(el: HTMLElement, cb: (e: MouseEvent) => void) {
	const state = {
		cb,
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
			state.cb(e);
		}
	};

	window.addEventListener("mouseup", onMouseUp);

	return {
		update(cb: (e: MouseEvent) => void) {
			state.cb = cb;
		},
		destroy() {
			window.removeEventListener("mousedown", onMouseDown);
			window.removeEventListener("mouseup", onMouseUp);
		},
	};
}
