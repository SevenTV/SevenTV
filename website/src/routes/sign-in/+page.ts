export function load({ url }) {
    return { redirect: url.searchParams.get("r") };
}
