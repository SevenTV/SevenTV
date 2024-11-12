import type { Role } from "$/gql/graphql";
import { getNumberFormatter } from "svelte-i18n";

export function priceFormat() {
    return getNumberFormatter({
        style: "currency",
        currency: "USD",
    });
}

export function numberFormat() {
    return getNumberFormatter({
        notation: "compact",
        unitDisplay: "narrow",
    });
}

export function isMobileLayout(): boolean {
    return window.matchMedia("screen and (max-width: 960px)").matches;
}

export function filterRoles(roles: Role[]) {
    return roles.filter((r) => r.name !== "Default").reverse();
}
