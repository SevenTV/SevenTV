import moment from "moment";

export function isXmasEvent() {
	return (
		moment().isAfter(moment("2025-12-13T00:00:00Z")) &&
		moment().isBefore(moment("2026-01-01T00:00:00Z"))
	);
}

export function isEasterEvent() {
	return (
		moment().isAfter(moment("2025-04-18T08:00:00Z").utc()) &&
		moment().isBefore(moment("2025-04-21T08:00:00Z").utc())
	);
}

export function isSummerGiftEvent() {
	return (
		moment().isAfter(moment("2025-07-27T00:00:00Z")) &&
		moment().isBefore(moment("2025-08-28T00:00:00Z"))
	);
}

export function isValentinesEvent() {
	return (
		moment().isAfter(moment("2026-02-13T09:00:00Z")) &&
		moment().isBefore(moment("2026-02-19T00:00:00Z"))
	);
}

