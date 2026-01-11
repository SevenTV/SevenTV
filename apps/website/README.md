# Website

This folder contains the **7TV website**.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for contribution guidelines.

## Flags

All country flags have been downloaded from [here](https://github.com/hampusborgos/country-flags).

---

## Development

### Install Dependencies

Make sure to have [NodeJS](https://nodejs.org/) and [pnpm](https://pnpm.io/) first.

Install all dependencies by using

```bash
pnpm i
```

This ensures your local environment has everything needed to run the project.

---

### Start Development Server

For running the website locally:

```bash
pnpm dev
```

Runs the website in **development mode**.  
Starts a local development server (usually available at ` http://localhost:4173/`) with hot reloading for instant updates on code changes.
Note that you can't login via API locally. You'll have to copy your session token from https://7tv.app/ `localStorage` named as `7tv-token` (After sign-in)
then add a `7tv-token` in `localStorage` and make the value to be your session token.

---

## Building

### Linting

```bash
pnpm eslint
```

Runs **ESLint** to check the project for syntax errors and code style issues.  
This helps maintain consistent and clean code quality.

---

### Build for Production

```bash
pnpm build
```

Compiles and optimizes the website for **production** deployment.  
Generates the final static files that can be hosted on a web server or deployed to a hosting platform.
