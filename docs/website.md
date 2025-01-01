# Website (Frontend)

The new website was developed along with the new v4 API.
It is written in SvelteKit as a single page application (SPA).
It aims to be a 1:1 replacement for the old website but with a modern design and more maintainable code.

## Components

The website is split into multiple components that are used to build the pages.
All components are located in the [`src/components`](../apps/website/src/components) directory.

## Pages

The website has multiple pages that are located in the [`src/routes`](../apps/website/src/routes) directory.

## Next Steps

### More Admin Tooling

Features like Mod Queue and Role Management are still missing from the new website.
Mods and admins are currently still using the old website for most of their work.

### Embeds (meta tags)

The new website is fully client side rendered (SPA) and doesnt have any server-side logic.
This makes it impossible to put meta tags in the response HTML for embeds shown on platforms like Discord and Twitter.
To fix this, the NGINX server which serves the static website files right now will have to be replace with a server that has some custom logic to inject the right meta tags into the response. (That's how it works on the old website)

### Show cosmetics for users that are not subscribed

Due to how our entitlement system works, the website currently does not show cosmetics to users that are not subscribed.
This is a problem because users who cancel their subscription think that they lost all their cosmetics when in reality they are still there.
To fix this, the v4 API should return a user's inventory including the cosmetics which are not directly reachable through the user graph node but rather through their subscription node.

### Virtual Scrolling

The new website currently doesn't have virtual scrolling.
When scrolling too far down the page, the browser will start to lag because it has to render all the elements in the list.
This is a problem especially when scrolling pages showing a lot of animated emotes.
Changing the [`emote-loader.svelte`](../apps/website/src/components/layout/emote-loader.svelte) component to use virtual scrolling would fix this on all pages.

### Internationalization (i18n)

The new website currently only supports English.
Work needs to be done to add support for other languages.
Some strings but not all are already extracted into the [`src/locales`](../apps/website/src/locales) directory, but the actual translation files are missing.
After that, to get help with translations, a new [Crowdin](https://crowdin.com/) project should be created to enable the community to help with translations. (This is how it worked on the old website)

### Emote Set Batch Operations

A highly requested feature is the ability to batch operations on emote sets.
This means that users should be able to remove or rename multiple emotes in a set at once and also copy entire emote sets.
Some frontend code for this is already implemented (commented out right now) but the backend code to make this possible is still missing.

### Other missing features and known bugs

The `#new-website-issues` channel on Discord has a list of other missing features and known bugs that need to be implemented/fixed.
