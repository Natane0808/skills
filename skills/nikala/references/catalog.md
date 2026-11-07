# Nikala UI Catalog

Authoritative inventory of registry items with the symbols each module exports.
Every export name in the tables below was verified against the live registry manifests
(v0.12.x), not copied from prose docs — several stale names in the upstream README and
`llms.txt` were corrected here (e.g. `DropzonePreviewList` → `DropzoneFileList`,
`TimelineIndicator` → `TimelinePoint`, `MessageAction` → `MessageActions`,
`ProgressIndicator` → `Progress`).

If an item is missing here, it may be newer than this snapshot — check
`bunx @nikala-ui/cli list --json` or the registry manifest at
`https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry/<item>.json`.

## Contents

- [Registry item types](#registry-item-types)
- [UI components](#ui-components)
- [Desktop components](#desktop-components)
- [Reactive hooks / primitives](#reactive-hooks--primitives)
- [Blocks](#blocks)
- [Choosing between similar items](#choosing-between-similar-items)

## Registry item types

Every manifest declares one of four `type` values, which determines the destination
directory chosen by the CLI:

| `type` | Added with | Lands in | Used as |
| :--- | :--- | :--- | :--- |
| `registry:ui` | `add <name>` | `alias.components` (`src/components/ui/`) | JSX components |
| `registry:hook` | `add --hook <name>` | `alias.hooks` (`src/hooks/`) | `create*` functions |
| `registry:block` | `add --block <name>` | components dir | full page sections |
| `registry:util` | (dependency) | `alias.utils` (`src/lib/`) | helpers like `cn` |

Manifest shape:

```json
{
  "name": "button",
  "title": "Button",
  "description": "An interactive button component with variant and size options.",
  "type": "registry:ui",
  "dependencies": ["clsx", "tailwind-merge", "class-variance-authority"],
  "registryDependencies": ["spinner"],
  "files": [
    { "path": "ui/button.tsx", "content": "...", "type": "registry:ui" }
  ]
}
```

`dependencies` are npm packages the CLI installs. `registryDependencies` are other
Nikala items the CLI installs first, recursively.

## UI components

Import path for all of these: `@/components/ui/<kebab-name>`.

| Item | Exports |
| :--- | :--- |
| `accordion` | `Accordion`, `AccordionItem`, `AccordionTrigger`, `AccordionContent` |
| `alert` | `Alert`, `AlertTitle`, `AlertDescription` |
| `api-table` | `ApiTable` |
| `aspect-ratio` | `AspectRatio` |
| `avatar` | `Avatar`, `AvatarImage`, `AvatarFallback` |
| `badge` | `Badge` |
| `banner` | `Banner` |
| `breadcrumb` | `Breadcrumb`, `BreadcrumbList`, `BreadcrumbItem`, `BreadcrumbLink`, `BreadcrumbPage`, `BreadcrumbSeparator`, `BreadcrumbEllipsis` |
| `bubble` | `Bubble` |
| `button` | `Button`, `buttonVariants` |
| `button-group` | `ButtonGroup` |
| `callout` | `Callout`, `CalloutTitle`, `CalloutDescription` |
| `card` | `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter` |
| `checkbox` | `Checkbox` |
| `code-block` | `CodeBlock` |
| `code-group` | `CodeGroup`, `CodeGroupList`, `CodeGroupTrigger`, `CodeGroupContent` |
| `collapsible` | `Collapsible`, `CollapsibleTrigger`, `CollapsibleContent` |
| `combobox` | `Combobox`, `ComboboxControl`, `ComboboxInput`, `ComboboxTrigger`, `ComboboxContent`, `ComboboxItem`, `ComboboxGroup` |
| `command` | `Command`, `CommandDialog`, `CommandInput`, `CommandList`, `CommandEmpty`, `CommandGroup`, `CommandItem`, `CommandFooter` |
| `component-viewer` | `ComponentViewer` |
| `container` | `Container` |
| `context-menu` | `ContextMenu`, `ContextMenuTrigger`, `ContextMenuContent`, `ContextMenuItem`, `ContextMenuCheckboxItem`, `ContextMenuRadioGroup`, `ContextMenuRadioItem`, `ContextMenuLabel`, `ContextMenuSeparator`, `ContextMenuShortcut`, `ContextMenuSub`, `ContextMenuSubTrigger`, `ContextMenuSubContent` |
| `dialog` | `Dialog`, `DialogTrigger`, `DialogOverlay`, `DialogContent`, `DialogHeader`, `DialogFooter`, `DialogTitle`, `DialogDescription`, `DialogClose` |
| `dropdown-menu` | `DropdownMenu`, `DropdownMenuTrigger`, `DropdownMenuContent`, `DropdownMenuItem`, `DropdownMenuLabel`, `DropdownMenuSeparator`, `DropdownMenuShortcut`, `DropdownMenuSub`, `DropdownMenuSubTrigger`, `DropdownMenuSubContent`, `DropdownMenuCheckboxItem`, `DropdownMenuRadioGroup`, `DropdownMenuRadioItem` |
| `dropzone` | `Dropzone`, `DropzoneIcon`, `DropzoneTitle`, `DropzoneDescription`, `DropzoneFileList`, `DropzoneFileItem`, `DropzoneRejectedItem` |
| `empty` | `Empty`, `EmptyIcon`, `EmptyTitle`, `EmptyDescription`, `EmptyAction` |
| `field` | `Field`, `FieldLabel`, `FieldDescription`, `FieldError` |
| `file-tree` | `FileTree`, `FileTreeFolder`, `FileTreeFile` |
| `footer` | `Footer`, `FooterContainer`, `FooterContent`, `FooterColumn`, `FooterColumnTitle`, `FooterColumnList`, `FooterLink`, `FooterBrand`, `FooterBottom`, `FooterCopyright`, `FooterSocials` |
| `form` | `Form` |
| `form-message` | `FormMessage` |
| `hover-card` | `HoverCard`, `HoverCardTrigger`, `HoverCardContent`, `HoverCardArrow` |
| `icon-button` | `IconButton` |
| `input` | `Input` |
| `input-group` | `InputGroup`, `InputGroupInput`, `InputGroupAddon` |
| `kbd` | `Kbd`, `KbdGroup` |
| `label` | `Label` |
| `list` | `List`, `ListGroup`, `ListHeader`, `ListItem` |
| `logo` | `Logo` |
| `marker` | `Marker`, `MarkerContent`, `MarkerDate`, `MarkerTyping` |
| `marquee` | `Marquee` |
| `message` | `Message`, `MessageAvatar`, `MessageHeader`, `MessageContent`, `MessageFooter`, `MessageActions`, `useMessage` |
| `navbar` | `Navbar`, `NavbarContainer`, `NavbarBrand`, `NavbarContent`, `NavbarItem`, `NavbarLink`, `NavbarActions`, `NavbarMobileToggle`, `NavbarMobileMenu`, `NavbarMobileLink`, `useNavbar` |
| `navigation-menu` | `NavigationMenu`, `NavigationMenuList`, `NavigationMenuItem`, `NavigationMenuTrigger`, `NavigationMenuContent`, `NavigationMenuLink` |
| `number-input` | `NumberInput` |
| `package-manager-tabs` | `PackageManagerTabs` |
| `pager` | `Pager`, `PagerLink` |
| `pagination` | `Pagination`, `PaginationContent`, `PaginationItem`, `PaginationLink`, `PaginationPrevious`, `PaginationNext`, `PaginationEllipsis` |
| `pin-input` | `PinInput`, `PinInputInput`, `PinInputLabel` |
| `popover` | `Popover`, `PopoverTrigger`, `PopoverContent` |
| `progress` | `Progress` (single component; style through the `class` prop) |
| `radio-group` | `RadioGroup`, `RadioGroupItem`, `RadioGroupItemLabel` |
| `rating` | `Rating` |
| `resizable` | `ResizableGroup`, `ResizablePanel`, `ResizableHandle` |
| `review-card` | `ReviewCard`, `ReviewHeader`, `ReviewProfile`, `ReviewAvatar`, `ReviewAuthor`, `ReviewRating`, `ReviewBody`, `ReviewFooter` |
| `rich-text-editor` | Multi-file item. Entry: `RichTextEditor`, plus `EditorToolbar`, `ToolbarButton`, `EditorBubbleMenu`, `EditorFooter`, `EditorLinkDialog`, `EditorImageDialog`, `EditorTableControls`, `EditorSlashCommand`, `useRichTextEditor`, `useEditor`, `getDefaultExtensions`, `htmlToMarkdown`, `markdownToHtml` |
| `scroll-area` | `ScrollArea` |
| `section-heading` | `SectionHeading` |
| `select` | `Select`, `SelectTrigger`, `SelectValue`, `SelectContent`, `SelectItem` |
| `separator` | `Separator` |
| `sheet` | `Sheet`, `SheetTrigger`, `SheetOverlay`, `SheetContent`, `SheetHeader`, `SheetFooter`, `SheetTitle`, `SheetDescription`, `SheetClose` |
| `sidebar` | `SidebarProvider`, `Sidebar`, `SidebarTrigger`, `SidebarRail`, `SidebarInset`, `SidebarHeader`, `SidebarContent`, `SidebarFooter`, `SidebarSeparator`, `SidebarGroup`, `SidebarGroupLabel`, `SidebarGroupAction`, `SidebarGroupContent`, `SidebarMenu`, `SidebarMenuItem`, `SidebarMenuButton`, `SidebarMenuAction`, `SidebarMenuBadge`, `SidebarMenuSkeleton`, `SidebarMenuSub`, `SidebarMenuSubItem`, `SidebarMenuSubButton`, `useSidebar` |
| `skeleton` | `Skeleton` |
| `slider` | `Slider`, `SliderTrack`, `SliderFill`, `SliderThumb`, `SliderLabel`, `SliderValueLabel` |
| `spinner` | `Spinner` |
| `stat` | `Stat`, `StatGroup`, `StatHeader`, `StatLabel`, `StatIcon`, `StatValue`, `StatUnit`, `StatTrend`, `StatHelpText` |
| `status` | `Status` |
| `steps` | `Steps`, `Step`, `StepTitle`, `StepDescription` |
| `switch` | `Switch` |
| `table` | `Table`, `TableHeader`, `TableBody`, `TableFooter`, `TableRow`, `TableHead`, `TableCell`, `TableCaption` |
| `table-of-contents` | `TableOfContents`, `Toc` |
| `tabs` | `Tabs`, `TabsList`, `TabsTrigger`, `TabsContent` |
| `textarea` | `Textarea` |
| `theme-manager` | `ThemeProvider`, `ThemeScript`, `ThemeToggle`, `useTheme` |
| `timeline` | `Timeline`, `TimelineItem`, `TimelineSeparator`, `TimelinePoint`, `TimelineConnector`, `TimelineContent`, `TimelineOppositeContent`, `TimelineTitle`, `TimelineDescription`, `TimelineTime` |
| `toast` | `Toast`, `ToastTitle`, `ToastDescription`, `ToastCloseButton`, `ToastRegion`, `ToastList`, `showToast` |
| `toggle` | `Toggle` |
| `toggle-group` | `ToggleGroup`, `ToggleGroupItem` |
| `tooltip` | `Tooltip`, `TooltipTrigger`, `TooltipContent`, `TooltipArrow` |

## Desktop components

Tauri v2–oriented; same install path (`@/components/ui/<name>`).

| Item | Exports |
| :--- | :--- |
| `titlebar` | `Titlebar`, `TitlebarControls`, `TitlebarTitle`, `TitlebarIcon`, `TitlebarActions`, `useTitlebar` |
| `titlebar-tabs` | `TitlebarTabs`, `TitlebarTabList`, `TitlebarTab`, `TitlebarTabAddButton`, `useTitlebarTabs` |
| `updater-modal` | `UpdaterModal` |

## Reactive hooks / primitives

Install with `--hook`. Import from `@/hooks/<kebab-name>`, call as camelCase.

| Hook file | Function | What it gives you |
| :--- | :--- | :--- |
| `create-active-element` | `createActiveElement` | currently focused element; `activeElement()`, `hasFocus()` |
| `create-app-updater` | `createAppUpdater` | Tauri v2 update lifecycle: `status()`, `progress()`, `checkForUpdates()`, `downloadAndInstall()`, `relaunch()` |
| `create-audio` | `createAudio` / `createVideo` | media playback: `isPlaying()`, `duration()`, `currentTime()`, `toggle()`, `seek()`, `setVolume()`. Registry item is named `create-audio` and its `title` is "createAudio & createVideo"; the source file is `create-audio-video.ts`. Note: this manifest ships `files: []`, so `add --hook create-audio` resolves metadata but writes nothing — if the hook is missing locally, pull it from `packages/hooks/src/create-audio-video.ts` or use `createAudio` via MCP `get_hook_code`. |
| `create-battery` | `createBattery` | battery level, charging state, time estimates |
| `create-chat-scroll` | `createChatScroll` | auto-scroll conversation until the user scrolls up |
| `create-click-outside` | `createClickOutside` | `{ target, onInteractOutside, enabled?, ignore? }` |
| `create-clipboard` | `createClipboard` | `copied()`, `copy(text)`, `error()` with auto-reset |
| `create-color-mode` | `createColorMode` | dark/light signal with system preference |
| `create-controllable-signal` | `createControllableSignal` | dual controlled/uncontrolled state pattern |
| `create-debounce` | `createDebounce` / `createThrottle` | rate-limit calls or signal updates |
| `create-disclosure` | `createDisclosure` | `isOpen()`, `open()`, `close()`, `toggle()` |
| `create-document-tabs` | `createDocumentTabs` | desktop tab model: pinned, dirty, wheel-scroll, `closeOthers()` |
| `create-document-title` | `createDocumentTitle` | set tab title, restore on unmount |
| `create-drop-zone` | `createDropZone` | drag-and-drop uploads, validation, native picker |
| `create-event-source` | `createEventSource` | SSE stream: `data()`, `status()`, `close()` |
| `create-favicon` | `createFavicon` | swap favicon from URL or SVG data URI |
| `create-fetch` | `createFetch` | `data()`, `loading()`, `error()`, `refetch()` |
| `create-focus-trap` | `createFocusTrap` | keep focus inside modals |
| `create-form` | `createForm` | field state, validation rules, errors, submit |
| `create-fullscreen` | `createFullscreen` | `isFullscreen()`, `toggle()` |
| `create-geolocation` | `createGeolocation` | GPS coords, accuracy, speed, heading |
| `create-global-shortcut` | `createGlobalShortcut` | OS-wide hotkeys via Tauri, web fallback |
| `create-hover` | `createHover` | hover state with enter/exit delays |
| `create-idle` | `createIdle` | inactivity detection |
| `create-infinite-scroll` | `createInfiniteScroll` | next-page triggers |
| `create-input-mask` | `createInputMask` | phone / card / date masks |
| `create-intersection-observer` | `createIntersectionObserver` | visibility ratios |
| `create-keybindings` | `createKeybindings` | shortcut combos, Escape handling |
| `create-lock-scroll` | `createLockScroll` | freeze background scroll under overlays |
| `create-long-press` | `createLongPress` | long press / touch hold |
| `create-media-query` | `createMediaQuery` | responsive breakpoint tracking |
| `create-mouse-position` | `createMousePosition` | window and element-relative coords |
| `create-network-status` | `createNetworkStatus` | online/offline + connection type |
| `create-orientation` | `createOrientation` | portrait/landscape + angle |
| `create-pagination` | `createPagination` | page ranges with ellipses |
| `create-permission` | `createPermission` | browser permission states |
| `create-previous` | `createPrevious` | previous value of an accessor |
| `create-resize-observer` | `createResizeObserver` | width/height/bounding rect |
| `create-scroll-into-view` | `createScrollIntoView` | smooth scroll to ref |
| `create-scroll-position` | `createScrollPosition` | offsets, direction, boundaries |
| `create-storage` | `createLocalStorage` / `createSessionStorage` | reactive storage with cross-tab sync |
| `create-tauri-window` | `createTauriWindow` | `isMaximized()`, `isMinimized()`, `isFullscreen()`, `isFocused()`, `minimize()`, `maximize()`, `toggleMaximize()`, `close()`, `startDragging()` |
| `create-timer` | `createTimer` | intervals, stopwatch, countdown |
| `create-tiptap-editor` | `createTiptapEditor` | headless TipTap binding: `editor()`, `isActive()`, `toggleBold()` |
| `create-undo-redo` | `createUndoRedo` | history stacks |
| `create-web-notification` | `createWebNotification` | desktop notifications + permission |
| `create-websocket` | `createWebSocket` | auto-reconnect, message parsing |
| `create-window-size` | `createWindowSize` | inner width/height |

## Blocks

Ready-made application and marketing sections — install with `--block`. These depend on
several components and hooks, so they are a fast way to get a coherent screen.

| Block | What it is |
| :--- | :--- |
| `hero-01` | landing hero section |
| `login-01` | sign-in screen |
| `register-01` | sign-up screen |
| `forgot-password-01` | password reset request screen |
| `otp-verification-01` | OTP / 2FA code entry |

## Choosing between similar items

- `Sheet` vs `Dialog`: Sheet is a slide-in panel from an edge (mobile drawers,
  settings); Dialog is a centered modal.
- `Pager` vs `Pagination`: `Pager` is simple prev/next; `Pagination` renders numbered
  pages with ellipsis.
- `InputGroup` vs `Field`: `InputGroup` attaches addons inside the input chrome;
  `Field` wraps label + description + error for form layout.
- `Button` vs `IconButton`: `IconButton` is square, icon-only, and requires an
  accessible label.
- `ToggleGroup` vs `RadioGroup`: ToggleGroup is non-exclusive multi-select; RadioGroup
  enforces one choice.
- `createDisclosure` vs `Collapsible`: same concept — use the hook when you need the
  state in multiple places, the component when it is purely presentational.
- `createColorMode` vs `theme-manager`: the hook gives you a raw dark/light signal; the
  component adds SSR anti-FOUC and persistence.
