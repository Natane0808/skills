# Theming: Tailwind CSS v4 Design Tokens

Nikala UI themes live entirely in CSS. There is no `tailwind.config.js` token block —
Tailwind v4 is configured through `@import "tailwindcss"` plus a CSS-first `@theme`
block. This reference explains the token contract, how the CLI mutates it, and how to
add dark mode and runtime switching correctly.

## Contents

- [The token contract](#the-token-contract)
- [Changing the theme with the CLI](#changing-the-theme-with-the-cli)
- [Writing your own tokens](#writing-your-own-tokens)
- [Dark mode](#dark-mode)
- [Runtime switching and anti-FOUC](#runtime-switching-and-anti-fouc)
- [Style extensions with cn()](#style-extensions-with-cn)
- [Common theming mistakes](#common-theming-mistakes)

## The token contract

The generated theme block maps semantic names to `--color-*` variables. Components
reference the semantic utilities, never the hex values:

```css
@import "tailwindcss";

@theme {
  --color-primary: #722f37;
  --color-primary-hover: #4a1c21;
  --color-surface: #1a1a1a;
  --color-surface-light: #2a2a2a;
  --color-text: #f5f5dc;
  --color-text-muted: #a0a0a0;
  /* background, card, border, muted, accent, ring, destructive ... */
}
```

Because Tailwind v4 derives utilities from `@theme` variables, defining
`--color-primary` automatically yields `bg-primary`, `text-primary`,
`border-primary`, `ring-primary`, and opacity modifiers like `bg-primary/90`.

Semantic vocabulary used across the system:

| Utility | Meaning | Typical use |
| :--- | :--- | :--- |
| `bg-background` / `text-foreground` | page canvas / body text | app shell |
| `bg-card` / `text-card-foreground` | raised surfaces | cards, popovers |
| `bg-muted` / `text-muted-foreground` | de-emphasized | captions, helper text |
| `bg-primary` / `text-primary-foreground` | brand accent | CTAs, active states |
| `bg-secondary` / `text-secondary-foreground` | low-emphasis buttons | secondary actions |
| `bg-accent` / `text-accent-foreground` | hover highlight | ghost button hover |
| `border-border` / `border-input` | strokes | dividers, field borders |
| `ring-ring` | focus ring | `focus-visible:ring-ring` |
| `destructive` | danger | delete, error |
| `bg-surface` / `bg-surface-light` | flat panels | desktop chrome, sidebars |

**Why semantic tokens matter:** a component written with `bg-card border-border`
re-themes instantly when the user flips accent or dark mode. A component written with
`bg-[#1a1a1a]` is permanently fixed and will look broken in every other theme.

## Changing the theme with the CLI

```bash
bunx @nikala-ui/cli theme                    # interactive selector
bunx @nikala-ui/cli theme set amber zinc     # <primary> <baseGray>
```

The command rewrites the tokens in the project CSS recorded in `nikala.config.json`
(`css` field). It does not touch component files — which is the point: tokens are the
single lever.

Primary accent options: `amber`, `red`, `orange`, `yellow`, `lime`, `green`, `emerald`,
`teal`, `cyan`, `sky`, `blue`, `indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose`,
`zinc`.

Base gray options: `zinc` (cool modern), `slate` (blue-tinted), `gray` (neutral),
`neutral` (warm), `stone` (earthy).

Pick accents by contrast against the base gray. `amber` on `stone` reads warm and
low-contrast; `amber` on `zinc` reads crisp — if the user wants a muted aesthetic,
prefer a warm base with a muted accent rather than lowering opacity on a bright one.

## Writing your own tokens

Add tokens to the same `@theme` block so they participate in Tailwind's utility
generation:

```css
@theme {
  --color-brand-ink: #2b2320;
  --color-brand-paper: #f7f1e6;
  --radius-card: 0.5rem;
  --font-display: "Cormorant Garamond", serif;
}
```

Then `bg-brand-paper`, `text-brand-ink`, `rounded-card`, `font-display` all work. For
dark mode, override the variable rather than declaring a second class set (see below).

Keep to the **`rounded-lg` ceiling** for cards, dialogs, and main containers. The design
system's restraint is deliberate; `rounded-xl` and above breaks the visual family.

## Dark mode

Override the same custom properties inside a `dark` selector so every semantic utility
follows automatically:

```css
@import "tailwindcss";

@custom-variant dark (&:where(.dark, .dark *));

@theme {
  --color-background: #ffffff;
  --color-foreground: #18181b;
  --color-card: #ffffff;
  --color-border: #e4e4e7;
  --color-primary: #722f37;
}

.dark {
  --color-background: #18181b;
  --color-foreground: #fafafa;
  --color-card: #27272a;
  --color-border: #3f3f46;
  --color-primary: #b5657a;
}
```

Two details that trip people up:

1. **`@custom-variant dark`** — Tailwind v4 needs to know what "dark" means. Declare it
   once; without it, `dark:` utilities may target `prefers-color-scheme` only and ignore
   the class-based toggle.
2. **Override the variable, not the utility** — redefining `--color-card` under `.dark`
   updates `bg-card` everywhere. Re-declaring `.dark .bg-card { ... }` is fragile and
   misses every derived utility.

Also lighten the primary in dark mode (`#722f37` → `#b5657a`): the accent that reads
well on white is too dark against `#18181b`.

## Runtime switching and anti-FOUC

Use the `theme-manager` component for the persistence + SSR story:

```tsx
import { ThemeProvider, ThemeScript, ThemeToggle } from "@/components/ui/theme-manager";
```

```html
<!-- Anti-FOUC: runs synchronously before paint -->
<head>
  <ThemeScript storageKey="nikala-theme" />
</head>
```

```tsx
<ThemeProvider storageKey="nikala-theme">
  <App />
</ThemeProvider>
```

`ThemeScript` must be in `<head>`, before `<ThemeProvider>`, and must not be deferred.
Otherwise the HTML paints with the default theme and then snaps to the stored one —
a visible flash on every load, and the classic "it works locally but flashes in
production" report.

For non-component code, `useTheme()` exposes the same state inside the provider tree.
If you only need a signal, `createColorMode` from `@/hooks/create-color-mode` gives you
`isDark` / `toggle` without markup.

## Style extensions with cn()

Extend a component's own classes through `class` + `cn()`, so `tailwind-merge` resolves
conflicts instead of letting CSS order decide:

```tsx
import { cn } from "@/lib/cn";

<Button class={cn("w-full", isActive() && "ring-2 ring-ring")}>Save</Button>
```

```ts
// src/lib/cn.ts — generated by init
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

Never concatenate class strings manually (`class={"w-full " + extra}`) — duplicate or
conflicting utilities then survive, and `tailwind-merge` cannot dedupe them.

## Common theming mistakes

| Symptom | Cause | Fix |
| :--- | :--- | :--- |
| Nothing is styled at all | CSS entry not imported, or `@tailwindcss/vite` missing | import the CSS file in the app entry; install the plugin |
| Some components styled, one hard-coded | component uses hex instead of tokens | replace with semantic utilities |
| Dark variants never apply | missing `@custom-variant dark` | declare it once in the CSS entry |
| Flash of wrong theme on load | `ThemeScript` placed after provider or deferred | move to `<head>`, synchronous |
| Theme change only affects new elements | token overridden via a nested selector with higher specificity | override the `--color-*` variable instead |
| `theme set` seems to do nothing | `css` path in `nikala.config.json` is stale | correct the `css` field, re-run |
