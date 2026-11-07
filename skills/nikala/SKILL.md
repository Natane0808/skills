---
name: nikala
description: Nikala UI framework skill for building SolidJS + Tailwind CSS v4 + Tauri v2 interfaces with the copy-paste Nikala UI component system (@nikala-ui/cli, @nikala-ui/core registry, @nikala-ui/mcp). Use this whenever the user mentions Nikala, nikala-ui, `nikala init`, `nikala add`, `bunx @nikala-ui/cli`, `@nikala-ui/*`, nikala.config.json, createClipboard / createTauriWindow / createDocumentTabs style primitives, or wants to build or scaffold a SolidJS UI, a Tauri desktop window chrome/titlebar, or a shadcn-style component registry project. Also use it when the user says "add a component" in a SolidJS project, asks about semantic Tailwind v4 design tokens (bg-background, text-foreground, bg-primary), or pastes Nikala component source that needs to follow splitProps / children() reactivity conventions — even if they never say "Nikala".
---

# Nikala UI — SolidJS + Tailwind v4 Component System

Nikala UI is **not** a runtime UI dependency. It is a copy-paste registry: the CLI
writes real `.tsx` / `.ts` source files into the user's project, and the project owns
them forever. This changes how you should reason about every task.

**Mental model — internalize this before writing code:**

| Instead of | Do this |
| :--- | :--- |
| `import { Button } from "some-ui-library"` | `bunx @nikala-ui/cli add button` → file lands at `src/components/ui/button.tsx` |
| Reading a component's props from npm types | Reading the actual installed source file, which can be edited |
| Waiting for an upstream release to fix an issue | Editing the local copy directly |
| Overriding styles via CSS specificity | Passing `class` and merging through `cn()` |

Because the code is local, **the ground truth is the file on disk**, not this document
or any changelog. Always read `src/components/ui/<name>.tsx` before assuming an API.

## Canonical facts

- Site / docs: <https://nikala.dev> — machine-readable manual: <https://nikala.dev/llms.txt>
- Repo: <https://github.com/nikala-ui/ui> — registry CDN:
  `https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry`
  (if that host is blocked on the network, the same `main` tree is mirrored at
  `https://cdn.jsdelivr.net/gh/nikala-ui/ui@main/packages/core/registry`)
- Current major line observed: **v0.12.x** (API in this skill matches that line)
- Package manager the project prefers: **bun** (falls back to pnpm / yarn / npm by lockfile)
- Peer stack: SolidJS ≥ 1.9, Tailwind CSS **v4** (`@import "tailwindcss"` + `@theme`), Tauri v2 for desktop
- Headless primitives underneath: **Kobalte** (`@kobalte/core`) and **Corvu** (`corvu`)
- Styling helpers: `clsx` + `tailwind-merge` via the generated `cn()` in `src/lib/cn.ts`
- Icons: `lucide-solid`

## Workflow: zero → working UI

### 1. Confirm the project is a fit

Nikala targets SolidJS. Verify before doing anything else — running `init` in a React
project produces a broken, unused setup:

```bash
# Expect solid-js in deps and a Solid entry (src/index.tsx or src/app.tsx for SolidStart)
cat package.json
```

If the project is React/Vue, stop and tell the user Nikala is SolidJS-only rather than
silently generating unusable files.

### 2. Initialize

```bash
bunx @nikala-ui/cli init           # interactive: dirs, base gray, primary accent
bunx @nikala-ui/cli init --defaults # non-interactive: src/components/ui, src/lib, zinc + amber
bunx @nikala-ui/cli init --ai      # also emit AI assistant rules (AGENTS.md, .cursorrules)
```

Non-interactive flags matter for automation. `--skip-dependencies` exists for
higher-level tooling that installs packages itself.

`init` performs six concrete actions — knowing these makes debugging easy:

1. Installs `clsx`, `tailwind-merge`, `class-variance-authority` (plus `tailwindcss`
   and `@tailwindcss/vite` if absent).
2. Writes path aliases into `tsconfig` / vite config.
3. Creates `src/lib/cn.ts`.
4. Generates the Tailwind v4 theme block and injects the CSS import into the entry file.
5. Writes `nikala.config.json`.
6. Optionally writes AI rules.

Resulting config — this is the contract every subsequent command reads:

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

If `init` output looks wrong (CSS not imported, `cn.ts` missing), fix the underlying
files rather than re-running init blindly over a customized project.

### 3. Add components, hooks, and blocks

```bash
# UI components (also desktop: titlebar, titlebar-tabs, updater-modal, theme-manager)
bunx @nikala-ui/cli add button card dialog
bunx @nikala-ui/cli add --all             # everything
bunx @nikala-ui/cli add button --overwrite

# Reactive hooks (note: -h / --hook, NOT a help flag)
bunx @nikala-ui/cli add --hook create-clipboard create-debounce
bunx @nikala-ui/cli add -h create-scroll-into-view

# Blocks: full marketing/app sections (hero-01, login-01, register-01, ...)
bunx @nikala-ui/cli add --block hero-01
```

`add` resolves `registryDependencies` transitively — adding `button` also pulls in
`spinner`, because the button imports `Spinner`. That is why deleting a "stray"
component can break a build.

Naming convention: component names are **kebab-case** (`dropdown-menu`); hooks are
**kebab-case** files (`create-click-outside.ts`) exporting **camelCase** functions
(`createClickOutside`). Getting this backwards is the most common mistake — a
`createClickOutside` argument to `add` still works, but `add --hook button` does not.

### 4. Discover what exists

```bash
bunx @nikala-ui/cli list                 # catalog + installed status
bunx @nikala-ui/cli list --installed
bunx @nikala-ui/cli list --component     # or --hook / --block
bunx @nikala-ui/cli list --json          # machine-readable, good for scripts
```

Or read the manifest directly — useful when the CLI output is unavailable:

```
https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry/<item>.json
```

See [references/catalog.md](references/catalog.md) for the full component / hook /
block inventory with exported symbol names, so you can pick the right item without
guessing. Load that reference whenever you need to choose a component by name.

### 5. Maintain

```bash
bunx @nikala-ui/cli diff button    # local vs registry, line-by-line
bunx @nikala-ui/cli upgrade        # update installed items to latest registry
bunx @nikala-ui/cli update --all
bunx @nikala-ui/cli remove button  # or: remove -h create-clipboard
bunx @nikala-ui/cli validate       # aka `doctor`: config, deps, CSS tokens
```

`diff` before `upgrade`. Because the project owns the source, an upgrade silently
discards local edits — surface the diff to the user before overwriting customized
files, and prefer `add <name> --overwrite` only with consent.

## Writing SolidJS code in a Nikala project

These rules are not style preferences. Each one prevents a specific, hard-to-debug
failure in SolidJS's fine-grained reactivity model.

### Never destructure props

```tsx
// ❌ Breaks reactivity: reads once at creation, never updates.
const { variant, class: className } = props;

// ✅ Required: splitProps keeps getters intact.
const [local, others] = splitProps(props, ["variant", "class"]);
```

Destructuring a SolidJS prop reads the signal accessor at that moment. The component
renders once and then goes stale — updates never propagate. This is why every
generated component opens with `splitProps`, and why your generated code must too.

### Memoize children before inspecting them

```tsx
// ✅ When iterating, counting, or conditionally rendering JSX children
const resolved = children(() => props.children);
```

`props.children` is a getter. Accessing it repeatedly during render creates new
reactive nodes each time; `children()` memoizes so tab hydration and conditional
branches stay stable.

### Guard SSR

```tsx
// ✅ Inside listeners / DOM access, so SolidStart SSR hydration does not crash
if (typeof window !== "undefined") { /* ... */ }
```

Generated hooks already do this — mirror it in code you write.

### Use semantic Tailwind v4 tokens, never raw colors

```tsx
// ❌ <div class="bg-[#1a1a1a] border-gray-700">
// ✅ <div class="bg-card border-border text-foreground">
```

Token vocabulary: `bg-background`, `text-foreground`, `bg-card`, `bg-muted`,
`text-muted-foreground`, `border-border`, `bg-primary`, `text-primary-foreground`,
`bg-secondary`, `bg-accent`, `ring-ring`, plus `destructive`. Hardcoded hex values
break dark mode and every theme the user selects. Style extensions go through `cn()`:

```tsx
<Button class={cn("w-full", isActive() && "ring-2 ring-ring")} />
```

### Border radius ceiling

The maximum radius across Nikala UI is **`rounded-lg`**. Do not introduce
`rounded-xl` / `-2xl` / `-3xl` on cards, dialogs, or main containers — it visually
breaks the system's deliberate restraint.

### Compose, don't hand-roll

Prefer existing primitives (`Button`, `Input`, `Badge`, `Progress`, `Tabs`, …) over
raw `<button>` / `<div>` constructs. If a primitive is missing, add it with the CLI
rather than approximating it.

### Use hooks from local aliases

```tsx
import { createClipboard } from "@/hooks/create-clipboard";
import { createDocumentTabs } from "@/hooks/create-document-tabs";
import { createTauriWindow } from "@/hooks/create-tauri-window";
```

Never import from `@nikala-ui/core` at runtime in an application — that package is the
internal registry source for the monorepo and docs site.

## Theming and dark mode

Tokens live in the project CSS (usually `src/index.css` or `src/app.css`) as a
CSS-first `@theme` block:

```css
@import "tailwindcss";

@theme {
  --color-primary: #722f37;
  --color-primary-hover: #4a1c21;
  --color-surface: #1a1a1a;
  --color-surface-light: #2a2a2a;
  --color-text: #f5f5dc;
  --color-text-muted: #a0a0a0;
  /* ...background/card/border/muted/ring tokens */
}
```

Change accents structurally, not by editing hex values ad hoc:

```bash
bunx @nikala-ui/cli theme                        # interactive
bunx @nikala-ui/cli theme set amber zinc         # primary accent + base gray
```

Primary choices include amber, red, orange, yellow, lime, green, emerald, teal, cyan,
sky, blue, indigo, violet, purple, fuchsia, pink, rose, zinc. Base grays: zinc, slate,
gray, neutral, stone.

For runtime theme switching use the `theme-manager` component (`ThemeProvider`,
`ThemeScript`, `ThemeToggle`, `useTheme`). In SSR, place `<ThemeScript storageKey="nikala-theme" />`
synchronously in `<head>` — before `<ThemeProvider>` — or the page flashes the wrong
theme on load.

Details and worked examples: [references/theming.md](references/theming.md).

## Desktop (Tauri v2)

Nikala is one of the few component systems with first-class desktop chrome:

```tsx
import { Titlebar, TitlebarControls, TitlebarTabs } from "@/components/ui/titlebar";
import { createDocumentTabs } from "@/hooks/create-document-tabs";

const tabs = createDocumentTabs({ initialTabs: [{ id: "App.tsx", title: "App.tsx" }] });

<Titlebar platform="macos" class="h-10 bg-card border-b">
  <TitlebarControls platform="macos" />
  <TitlebarTabs manager={tabs} variant="editor" onAddTab={() => tabs.addTab({ id: "New.tsx", title: "New.tsx" })} />
</Titlebar>
```

`platform` must match the running OS (`macos` positions traffic lights left; Windows /
Linux use right-side controls). Pair with `createTauriWindow` for `isMaximized()`,
`toggleMaximize()`, `startDragging()`, and `updater-modal` + `createAppUpdater` for
in-app updates. Read [references/desktop.md](references/desktop.md) for the setup guide,
window permissions, and the updater flow.

## AI / MCP integration

If the user works in Cursor, Claude Desktop, or Antigravity, point them at the registry
server so the editor fetches live source instead of hallucinating:

```json
{ "mcpServers": { "nikala-ui": { "command": "npx", "args": ["-y", "@nikala-ui/mcp"] } } }
```

Remote SSE endpoint: `https://nikala.dev/api/mcp/sse`. Tools include
`list_components`, `list_hooks`, `get_component_code`, `get_hook_code`,
`install_component`, `install_hook`, `validate_project`, `inspect_workspace`,
`search_docs`. Also run `bunx @nikala-ui/cli init --ai` (or `nikala rules`) to write
project-local rule files. See [references/mcp.md](references/mcp.md).

## Troubleshooting

Symptoms, causes, and fixes are collected in [references/troubleshooting.md](references/troubleshooting.md).
The high-frequency ones:

- **Import fails after `add`** → alias not configured; `@/components/ui/x` must resolve
  per `nikala.config.json` and `tsconfig.paths`. Run `nikala validate`.
- **Styles are all missing** → the CSS entry was never imported in the app entry, or
  Tailwind v4 plugin (`@tailwindcss/vite`) is absent.
- **Component renders once and never updates** → props were destructured; use `splitProps`.
- **`-h` behaves like help** → `-h` is `--hook` on `add`; use `--help` for help.
- **Dark mode does not switch** → tokens are hardcoded hex, or `ThemeScript` is misplaced.

## Working agreement

- Read the installed component source before documenting or extending its API.
- When adding a component that another component imports, add both (the CLI does this;
  if you install by hand, replicate `registryDependencies`).
- Do not hand-author files under `src/components/ui/` that the CLI could generate — it
  guarantees the `splitProps` / token / SSR conventions for free.
- Before upgrading, run `diff` and show the user what local edits would be lost.
- Prefer `bun`/`bunx` in commands; adapt only if the project lockfile says otherwise.
