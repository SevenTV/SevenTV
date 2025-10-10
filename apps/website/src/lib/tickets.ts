import { writable } from "svelte/store";

function persistentWritable<T>(key: string, initialValue: T) {
	const storedValue = localStorage.getItem(key);
	const value = storedValue ? JSON.parse(storedValue) : initialValue;

	const store = writable<T>(value);

	store.subscribe((val) => {
		localStorage.setItem(key, JSON.stringify(val));
	});

	return store;
}
export const totalPublicRequests = persistentWritable("totalPublicRequests", 0);
export const totalPersonalRequests = persistentWritable("totalPersonalRequests", 0);
export const totalReportRequests = persistentWritable("totalReportRequests", 0);
export const countriesFilter = persistentWritable<string[]>("countriesFilter", []);
export const galleryTicketMode = persistentWritable("galleryTicketMode", false);
export const expandedTickets = persistentWritable("expandedTickets", false);
export const refetchRequested = writable(false);
