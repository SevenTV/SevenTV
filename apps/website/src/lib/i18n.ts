import { browser } from "$app/environment";
import moment from "moment/min/moment-with-locales";
import { init, locale, register } from "svelte-i18n";

const defaultLocale = "en";

// This can't be part of the locale files because it must be always loaded
export const localeNames: { [key: string]: string } = {
	ar: "Arabic",
	cs: "Czech",
	de: "German",
	el: "Greek",
	en: "English",
	"en-GB": "English (UK)",
	es: "Spanish",
	"es-419": "Spanish (Latin)",
	fi: "Finnish",
	fr: "French",
	lt: "Lithuanian",
	nl: "Dutch",
	pl: "Polish",
	"pt-BR": "Portuguese (Brazil)",
	"pt-PT": "Portuguese (Portugal)",
	ru: "Russian",
	sk: "Slovak",
	"sv-SE": "Swedish",
	tr: "Turkish",
	ua: "Ukrainian",
	"zh-CN": "Chinese (Simplified)",
	"zh-TW": "Chinese (Traditional)",
};

// https://developer.crowdin.com/language-codes
register("ar", () => import("../locales/ar.json"));
register("cs", () => import("../locales/cs.json"));
register("de", () => import("../locales/de.json"));
register("el", () => import("../locales/el.json"));
register("en", () => import("../locales/en.json"));
register("en-GB", () => import("../locales/en-GB.json"));
register("es", () => import("../locales/es.json"));
register("es-419", () => import("../locales/es-419.json"));
register("fi", () => import("../locales/fi.json"));
register("fr", () => import("../locales/fr.json"));
register("lt", () => import("../locales/lt.json"));
register("nl", () => import("../locales/nl.json"));
register("pl", () => import("../locales/pl.json"));
register("pt-BR", () => import("../locales/pt-BR.json"));
register("pt-PT", () => import("../locales/pt-PT.json"));
register("ru", () => import("../locales/ru.json"));
register("sk", () => import("../locales/sk.json"));
register("sv-SE", () => import("../locales/sv-SE.json"));
register("tr", () => import("../locales/tr.json"));
register("ua", () => import("../locales/ua.json"));
register("zh-CN", () => import("../locales/zh-CN.json"));
register("zh-TW", () => import("../locales/zh-TW.json"));

function getInitialLocale() {
	if (browser) {
		const savedLocale = window.localStorage.getItem("locale");
		if (savedLocale) {
			return savedLocale;
		}
	}
	return defaultLocale;
}

locale.subscribe((l) => {
	if (browser && l) {
		window.localStorage.setItem("locale", l);
		moment.locale(l);
	}
});

export function initI18n() {
	init({
		initialLocale: getInitialLocale(),
		fallbackLocale: defaultLocale,
	});
}

initI18n();
