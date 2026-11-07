# Troubleshooting & Configuration Reference

Diagnose by symptom. Most Nikala failures come from four layers: **CLI setup**,
**alias resolution**, **Tailwind token wiring**, and **SolidJS reactivity**. Identify the
layer first, then apply the fix — patching the wrong layer is how projects end up with
duplicated config.

## Contents

- [Diagnostic ladder](#diagnostic-ladder)
- [CLI and init problems](#cli-and-init-problems)
- [Alias and import problems](#alias-and-import-problems)
- [Styling and token problems](#styling-and-token-problems)
- [Reactivity problems](#reactivity-problems)
- [Diff / upgrade / remove](#diff--upgrade--remove)
- [Custom registries](#custom-registries)
- [Config reference](#config-reference)

## Diagnostic ladder

Run in this order; each step narrows the layer:

```bash
cat nikala.config.json                 # 1. is the project configured at all?
bunx @nikala-ui/cli validate           # 2. config + deps + CSS tokens check
bunx @nikala-ui/cli list --installed   # 3. what is actually on disk?
cat src/lib/cn.ts                      # 4. does the helper exist?
ls src/components/ui                   # 5. files the config claims vs files present
```

`validate` (alias `doctor`) is the intended first stop. It reports missing config,
missing dependencies, and missing CSS design tokens.

## CLI and init problems

**`init` ran in the wrong directory.** It writes into `process.cwd()`. Run it from the
project root (where `package.json` lives). If it created a stray `src/` somewhere else,
delete that tree rather than mixing artifacts.

**`init` did not install dependencies.** It detects the package manager from lockfiles
(`bun.lock`/`bun.lockb` → bun, then pnpm, yarn, npm; default bun). If install failed, run
manually:

```bash
bun add clsx tailwind-merge class-variance-authority tailwindcss @tailwindcss/vite
```

**`-h` printed help instead of adding hooks.** On `add`, `-h` means `--hook`. Use
`--help` for help.

**`add --hook button` fails.** Components and hooks are separate namespaces. Use `add
button` for components, `add --hook create-clipboard` for hooks.

**A component imports something that does not exist.** `add` resolves
`registryDependencies` transitively, but manual file copying does not. Check the item
manifest's `registryDependencies` and add those too (e.g. `button` needs `spinner`).

**Non-interactive init hangs.** Pass `--defaults`. Add `--skip-dependencies` when another
process manages installs.

## Alias and import problems

Canonical imports:

```tsx
import { Button } from "@/components/ui/button";
import { createClipboard } from "@/hooks/create-clipboard";
import { cn } from "@/lib/cn";
```

If `@/...` does not resolve, `init`'s alias configuration either failed or was later
overwritten. Both sides must agree:

```json
// tsconfig.json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./src/*"] }
  }
}
```

```ts
// vite.config.ts (Solid)
import path from "node:path";
export default defineConfig({ resolve: { alias: { "@": path.resolve(__dirname, "src") } } });
```

The directories in `nikala.config.json` (`alias.components`, `alias.utils`,
`alias.hooks`) are where the CLI **writes**; `@/` is how code **reads**. If you moved
components into `src/ui`, update `alias.components` — otherwise new `add` calls write to
the old location and imports diverge.

**Never import `@nikala-ui/core` in application code.** It is the monorepo's internal
registry source. Applications consume local copies only.

## Styling and token problems

**Everything is unstyled.** In order of likelihood:

1. The CSS entry was never imported in the app entry (`import "./index.css";`).
2. `@tailwindcss/vite` is not installed, or not registered in `vite.config.ts`.
3. The CSS file contains only `@import "tailwindcss";` with no `@theme` tokens, so
   `bg-card`, `border-border`, etc. generate nothing.

**Utilities exist but render transparent.** The token is defined under a wrong name.
Tailwind v4 requires `--color-<name>`; `--card: #fff` produces no `bg-card`.

**Theme changes do not apply.** `theme set` writes to the file in `nikala.config.json`'s
`css` field. If that path is stale (file moved/renamed), the command edits a file nothing
imports. Note `init` picks `src/app.css` for SolidStart or when it exists, else
`src/index.css`.

**Dark mode ignored.** Declare the variant once:

```css
@custom-variant dark (&:where(.dark, .dark *));
```

Then override the `--color-*` variables inside `.dark` (see
[theming.md](theming.md#dark-mode)).

**Wrong theme flashes on first paint.** `<ThemeScript storageKey="nikala-theme" />` must
be synchronous in `<head>` before `<ThemeProvider>`.

**Radius looks off-system.** `rounded-xl`+ is out of bounds; the ceiling is `rounded-lg`.

## Reactivity problems

**A prop change never re-renders the component.**

```tsx
const { class: className, variant } = props;   // ❌ read once, frozen
const [local, rest] = splitProps(props, ["class", "variant"]);  // ✅
```

This is the single most common custom-code bug in SolidJS. If UI "stops updating" only
after a user action, suspect destructuring.

**Children behave inconsistently in tabs/conditionals.** Memoize:

```tsx
const resolved = children(() => props.children);
```

**Hydration mismatch / crash under SolidStart SSR.** Accessing `window`, `document`, or
`localStorage` during render. Guard:

```tsx
if (typeof window !== "undefined") { /* ... */ }
```

Prefer `onMount` / `onCleanup` over imperative top-level DOM code.

**Effects never clean up.** Return cleanup from `onCleanup`, especially in hooks copied
from the registry and then modified — a removed `onCleanup` leaks listeners across route
changes.

**Subscription fires twice.** SolidStart renders server-side then hydrates; top-level
side effects run in both passes. Move subscriptions into `onMount`.

## Diff / upgrade / remove

```bash
bunx @nikala-ui/cli diff button         # local vs registry
bunx @nikala-ui/cli diff                # pick from installed items
bunx @nikala-ui/cli upgrade button      # or `update`
bunx @nikala-ui/cli upgrade --all
bunx @nikala-ui/cli remove button
bunx @nikala-ui/cli remove -h create-clipboard
bunx @nikala-ui/cli remove --all
```

Upgrading **overwrites local files**. Because the copy-paste model invites customization,
always `diff` first and show the user which of their edits would be lost. If they need
both, upgrade into a scratch branch and re-apply edits manually.

`remove` asks for confirmation before deleting; it does not prune now-unused npm
dependencies — clean those up separately.

## Custom registries

`add` accepts three source forms:

```bash
bunx @nikala-ui/cli add @acme/hero-01                       # namespaced private registry
bunx @nikala-ui/cli add https://example.com/r/card.json     # direct manifest URL
bunx @nikala-ui/cli add card                                 # official registry
```

Namespaced registries are declared in `nikala.config.json`:

```json
{
  "registries": { "@acme": "https://registry.example.com/json" }
}
```

Resolution order: direct URL → namespaced registry → local package registry →
official GitHub CDN
(`https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry/<name>.json`).
A custom manifest must expose `name` and a `files` array or the CLI rejects it.

## Config reference

`nikala.config.json` produced by `init --defaults`:

```json
{
  "$schema": "https://nikala.dev/schema.json",
  "style": "default",
  "baseColor": "zinc",
  "primaryColor": "amber",
  "css": "src/index.css",
  "alias": {
    "components": "src/components/ui",
    "utils": "src/lib",
    "hooks": "src/hooks"
  }
}
```

| Field | Meaning | Change it when |
| :--- | :--- | :--- |
| `style` | component style preset | the project standardizes on a non-default preset |
| `baseColor` | gray ramp | the brand leans warm/cool (also `theme set`) |
| `primaryColor` | accent | rebranding (also `theme set`) |
| `css` | entry stylesheet with `@theme` | CSS file was moved or renamed |
| `alias.components` | where UI files are written | custom directory layout |
| `alias.utils` | where `cn.ts` lives | custom directory layout |
| `alias.hooks` | where hook files are written | custom directory layout |
| `registries` | private registry map | using a company registry |

After changing `alias.*`, move existing files to match, or imports break.
