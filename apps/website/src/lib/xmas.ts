import moment from "moment";

export function isXmasEvent() {
	return (
		moment().isAfter(moment("2024-12-14T00:00:00Z")) &&
		moment().isBefore(moment("2024-12-31T00:00:00Z"))
	);
}
