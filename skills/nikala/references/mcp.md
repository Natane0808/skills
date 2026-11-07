# MCP, AI Rules, and Editor Integration

Nikala UI ships an official MCP server so AI editors read *real* registry source instead
of guessing component APIs. This reference covers setting it up, the available tools and
resources, and the project-local AI rules file.

## Contents

- [Why this matters](#why-this-matters)
- [Stdio setup](#stdio-setup)
- [Remote SSE setup](#remote-sse-setup)
- [Tools](#tools)
- [Bundled prompts](#bundled-prompts)
- [Resources](#resources)
- [Project-local AI rules](#project-local-ai-rules)
- [Using MCP without a server](#using-mcp-without-a-server)

## Why this matters

Nikala components are copied into the project and often edited. A model that recalls a
component API from training data may describe a version that no longer matches the file on
disk. The MCP server sidesteps this: `get_component_code` returns the current manifest,
and `inspect_workspace` returns what is actually installed locally.

Practical rule: **if the MCP server is available, prefer it over memory; if it is not,
read the local file.** Both beat guessing.

## Stdio setup

Cursor, Claude Desktop, Antigravity, Windsurf:

```json
{
  "mcpServers": {
    "nikala-ui": {
      "command": "npx",
      "args": ["-y", "@nikala-ui/mcp"]
    }
  }
}
```

Config file locations differ per client (`.cursor/mcp.json`, `claude_desktop_config.json`,
etc.). Restart the client after editing — most do not hot-reload MCP config.

## Remote SSE setup

For clients that support remote MCP endpoints:

- SSE URL: `https://nikala.dev/api/mcp/sse`
- Messages URL: `https://nikala.dev/api/mcp/messages`

Remote transport avoids a local Node process and always serves the latest registry, at
the cost of requiring network access.

## Tools

| Tool | Purpose | Arguments |
| :--- | :--- | :--- |
| `list_components` | list registry UI components | none |
| `list_hooks` | list reactive primitives | none |
| `get_component_code` | full TSX source for a component | `{ "name": "button" }` |
| `get_hook_code` | full TS source for a hook | `{ "name": "create-clipboard" }` |
| `install_component` | write TSX into `src/components/ui/` | `{ "name": "button", "workspace_dir"?: "/path" }` |
| `install_hook` | write TS into `src/hooks/` | `{ "name": "create-audio", "workspace_dir"?: "/path" }` |
| `validate_project` | check `nikala.config.json`, `cn.ts`, dirs | `{ "workspace_dir"?: "/path" }` |
| `inspect_workspace` | report which components/hooks are installed | `{ "workspace_dir"?: "/path" }` |
| `search_docs` | fuzzy search components, hooks, guidelines | `{ "query": "dialog modal" }` |

`install_*` tools write files directly — equivalent to `nikala add`. They do **not**
install npm dependencies; add those separately if the manifest requires them. Prefer the
CLI when you also want transitive `registryDependencies` resolution automatically.

## Bundled prompts

- `create_form_page` — full SolidJS form with Input, Button, Card, and `createForm`.
- `setup_theme_provider` — `ThemeProvider`, `ThemeToggle`, anti-FOUC `ThemeScript`.
- `create_audio_player` — media player UI from `Progress`, `Button`, `Badge`, `createAudio`.

Use these as starting scaffolds when the user asks for exactly that shape; they encode the
project's own conventions.

## Resources

- `nikala://rules/solidjs` — reactivity guidelines: `splitProps`, fine-grained signals,
  `children()` memoization.
- `nikala://rules/theming` — Tailwind v4 tokens, anti-FOUC `ThemeScript`, `rounded-lg`
  ceiling.

## Project-local AI rules

```bash
bunx @nikala-ui/cli init --ai   # during setup
bunx @nikala-ui/cli rules       # later (alias: `ai`)
```

Writes `.cursor/rules/nikala.mdc`, `.cursorrules`, and `AGENTS.md` so any agent in the
repo inherits the same constraints (no prop destructuring, semantic tokens, hook import
paths, radius ceiling). These files are additive — commit them so every collaborator's
assistant behaves consistently. Re-run after upgrading Nikala so the rules match the new
registry.

If the repo already has an `AGENTS.md`, review the merge: the generator may append its
section, and duplicated or contradictory guidance confuses agents more than no guidance.

## Using MCP without a server

When MCP is unavailable, you can still get authoritative data over plain HTTP:

```bash
# catalog
curl https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry/index.json

# single item manifest (includes full source in `files[].content`)
curl https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry/button.json

# machine-readable manual for the whole system
curl https://nikala.dev/llms.txt
```

Or locally:

```bash
bunx @nikala-ui/cli list --json
cat src/components/ui/button.tsx
```

`scripts/fetch-item.mjs` in this skill automates fetching and summarising a manifest;
see `--help` in that file.
