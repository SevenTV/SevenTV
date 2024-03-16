import { browser } from "$app/environment";
import moment from "moment/min/moment-with-locales";
import { getLocaleFromNavigator, init, locale, register } from "svelte-i18n";

const defaultLocale = "en";

// This can't be part of the locale files because it must be always loaded
export const localeNames: { [key: string]: string } = {
	en: "English",
};

// https://developer.crowdin.com/language-codes
register("en", () => import("../locales/en.json"));

function getInitialLocale() {
	if (!browser) {
		return defaultLocale;
	}
	const savedLocale = window.localStorage.getItem("locale");
	return savedLocale ?? getLocaleFromNavigator();
}

locale.subscribe((l) => {
	if (browser && l) {
		window.localStorage.setItem("locale", l);
		moment.locale(l);
	}
});

init({
	initialLocale: getInitialLocale(),
	fallbackLocale: defaultLocale,
});
