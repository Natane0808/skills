# Desktop Suite (Tauri v2)

Nikala ships desktop-specific components and hooks that most copy-paste libraries lack:
frameless window chrome, macOS traffic lights, multi-document tabs, OS-wide hotkeys, and
an auto-updater UI. Use this reference when the project is a Tauri v2 app or the user
asks for a titlebar, window controls, tabs, global shortcuts, or in-app updates.

## Contents

- [Prerequisites](#prerequisites)
- [Frameless window + titlebar](#frameless-window--titlebar)
- [Titlebar tabs](#titlebar-tabs)
- [Window state with createTauriWindow](#window-state-with-createtauriwindow)
- [Global shortcuts](#global-shortcuts)
- [Auto-updater](#auto-updater)
- [Platform differences](#platform-differences)
- [Permissions and pitfalls](#permissions-and-pitfalls)

## Prerequisites

```bash
bunx @nikala-ui/cli init
bunx @nikala-ui/cli add titlebar titlebar-tabs updater-modal button progress
bunx @nikala-ui/cli add -h create-document-tabs create-tauri-window create-app-updater create-global-shortcut
```

Tauri capabilities needed (in `src-tauri/capabilities/default.json`):

```json
{
  "permissions": [
    "core:window:allow-start-dragging",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-is-maximized",
    "core:window:allow-set-fullscreen"
  ]
}
```

Add `global-shortcut:allow-register` / `allow-unregister` only if you use
`createGlobalShortcut`, and the `updater:*` permissions only if you use the updater.
Granting everything up front makes the security surface larger than it needs to be.

The window must be **frameless** for custom chrome to make sense:

```json
{
  "app": {
    "windows": [{ "title": "App", "decorations": false, "transparent": false }]
  }
}
```

With `decorations: true` you get a native titlebar *and* your custom one — a doubled
chrome that looks broken.

## Frameless window + titlebar

```tsx
import { Titlebar, TitlebarControls } from "@/components/ui/titlebar";

<Titlebar platform="macos" class="h-10 bg-card border-b">
  <TitlebarControls platform="macos" />
  <span class="text-sm text-muted-foreground ml-2">My App</span>
</Titlebar>
```

`Titlebar` handles the drag region (`data-tauri-drag-region`) so the window can be moved
by its header. Interactive children (buttons, tabs) are excluded from dragging — if you
add your own controls, place them inside `TitlebarActions` or mark them so they are not
treated as drag surface.

`platform` is not decoration: `macos` reserves left inset for traffic lights and renders
them; `windows` / `linux` render right-aligned controls. Hardcoding `macos` on a
Windows build leaves dead space and misplaces controls.

## Titlebar tabs

`TitlebarTabs` consumes a tab manager from `createDocumentTabs`. The hook owns state;
the component only renders it.

```tsx
import { Titlebar, TitlebarControls, TitlebarTabs } from "@/components/ui/titlebar";
import { createDocumentTabs } from "@/hooks/create-document-tabs";
import { FileCode, Terminal } from "lucide-solid";

const tabs = createDocumentTabs({
  initialTabs: [
    { id: "main.rs", title: "main.rs", icon: Terminal, isPinned: true },
    { id: "app.tsx", title: "App.tsx", icon: FileCode, isDirty: true },
  ],
  enableKeybindings: true, // Ctrl+W / Cmd+W closes the active tab
});

<Titlebar platform="macos" class="h-10 bg-card border-b">
  <TitlebarControls platform="macos" class="mr-2" />
  <TitlebarTabs
    manager={tabs}
    variant="editor"
    onAddTab={() => tabs.addTab({ id: `doc-${tabs.count() + 1}.ts`, title: "Untitled", icon: FileCode })}
  />
</Titlebar>
```

Manager API: `tabs()`, `activeTabId()`, `addTab()`, `closeTab()`, `markDirty()`,
`togglePin()`, `closeOthers()`, `count()`. `variant` selects the visual treatment
(`pills` for app-like tabs, `editor` for code-editor tabs).

Persist `isDirty` honestly. Closing an unsaved tab that reports clean loses user work;
that is the one behavior in a desktop app users will not forgive.

## Window state with createTauriWindow

```tsx
import { createTauriWindow } from "@/hooks/create-tauri-window";

const win = createTauriWindow();

win.isMaximized();       // Accessor<boolean>
win.isFocused();
win.minimize();
win.toggleMaximize();
win.close();
win.startDragging();
```

If you need a custom control strip instead of `TitlebarControls`, drive it from these
signals so the maximize icon reflects real state rather than a local guess (the user can
also maximize via OS shortcuts, which your local signal would miss).

Outside Tauri these calls no-op rather than throw, so components degrade gracefully in a
browser preview.

## Global shortcuts

```tsx
import { createGlobalShortcut } from "@/hooks/create-global-shortcut";

createGlobalShortcut({
  shortcut: ["CommandOrControl+Shift+K", "Alt+Space"],
  onTrigger: (key) => console.log("Global hotkey fired:", key),
});
```

`CommandOrControl` maps to Cmd on macOS and Ctrl elsewhere — prefer it over hardcoding
`Cmd`. Registration goes through the Tauri plugin with a web keyboard fallback, so the
same call works in a browser tab (without OS-wide reach). Choose combos that are not
already reserved by the OS; `Alt+Space` is the Windows system menu on many setups.

## Auto-updater

`UpdaterModal` renders the whole update flow (checking, available, downloading with
progress, ready-to-relaunch) and reads from `createAppUpdater`.

```tsx
import { createSignal } from "solid-js";
import { UpdaterModal } from "@/components/ui/updater-modal";
import { createAppUpdater } from "@/hooks/create-app-updater";
import { Button } from "@/components/ui/button";

export function AppUpdater() {
  const [isOpen, setIsOpen] = createSignal(false);

  const updater = createAppUpdater({
    autoCheck: true,
    currentVersion: "v1.0.0",
    onUpdateAvailable: () => setIsOpen(true),
  });

  return (
    <>
      <Button onClick={() => updater.checkForUpdates()}>Check for Updates</Button>
      <UpdaterModal
        open={isOpen()}
        onOpenChange={setIsOpen}
        updater={updater}
        appName="My App"
      />
    </>
  );
}
```

Updater API: `status()`, `progress()`, `updateInfo()`, `checkForUpdates()`,
`downloadAndInstall()`, `relaunch()`.

`currentVersion` should come from the real build version (Tauri `getVersion()`), not a
hardcoded literal — a stale literal makes every launch look like an update is available.

## Platform differences

| Concern | macOS | Windows / Linux |
| :--- | :--- | :--- |
| Control placement | traffic lights, left | buttons, right |
| Titlebar inset | needs left spacing | needs right spacing |
| Drag region | whole header | whole header |
| Resize affordance | edges | edges, often needs explicit resize handles |
| Shortcut modifier | `CommandOrControl` → Cmd | → Ctrl |

Test both layouts, or accept the OS the user actually targets — but never ship a
`platform="macos"` header in a Windows-only build.

## Permissions and pitfalls

- **Window does not drag** → `decorations: false` set but `core:window:allow-start-dragging`
  missing, or your header element is not the drag region.
- **Controls do nothing** → missing window permissions; Tauri v2 denies silently at the
  capability layer.
- **Double titlebar** → native decorations still enabled.
- **Maximize icon desyncs** → local boolean instead of `win.isMaximized()`.
- **Global hotkey never fires** → `global-shortcut` permission missing, or the combo is
  reserved by the OS.
- **Updater loops** → `currentVersion` hardcoded and never bumped.
