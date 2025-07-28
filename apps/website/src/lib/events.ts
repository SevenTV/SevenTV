import moment from "moment";

export function isXmasEvent() {
	return (
		moment().isAfter(moment("2024-12-14T00:00:00Z")) &&
		moment().isBefore(moment("2024-12-27T00:00:00Z"))
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

