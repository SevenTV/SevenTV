# Contribution

## Developing

First, please install [pnpm](https://pnpm.io/) if you haven't already.

To install all dependencies, run `pnpm install` in this folder.

Once all dependencies are installed, you can start the development server:

```bash
pnpm run dev
```

## Building

To create a production version of the app:

```bash
pnpm run build
```

You can preview the production build with `pnpm run preview`.

## Guidelines

The following guidelines are not strict rules. Try to follow them as much as possible.

### No unnecessary dependencies

Whenever possible, avoid introducing new dependencies. Most things you want to do (especially in HTML/CSS) can be done without any dependencies just by writing a few lines more.

### Prefer HTML + CSS over JS solutions

Don't throw JS at everything. If you can solve a problem with HTML and CSS only, prefer this solution.

A good example for this rule is the FAQ section on the subscription store page. See [faq.svelte](./src/components/store/faq.svelte).

### Avoid inline styles

Try to avoid inline styles. If you need to style a component, create a new class. We don't want our project to look like a Tailwind mess.

### Use `kebab-case` for class names

Use [`kebab-case`](https://developer.mozilla.org/en-US/docs/Glossary/Kebab_case) for class names.

### Use `const` for all immutable variables

Use `const` for all variables that are not reassigned. This lets the JS engine further optimize the code.
