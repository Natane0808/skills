# Skills

[![skills.sh](https://skills.sh/b/Natane0808/skills)](https://skills.sh/Natane0808/skills)

A collection of agent skills — reusable, task-specific instruction packages that an AI
coding agent loads when a matching task appears.

Each skill lives in its own directory under [`skills/`](skills/) and follows the
`SKILL.md` + `references/` + `scripts/` structure.

## Available skills

| Skill | Description | Install |
| :--- | :--- | :--- |
| [`nikala`](skills/nikala/) | Build SolidJS + Tailwind CSS v4 + Tauri v2 interfaces with the [Nikala UI](https://nikala.dev) copy-paste component system — CLI workflow, verified component/hook catalog, design tokens, desktop suite, and troubleshooting. | `npx skills add Natane0808/skills --skill nikala` |
| [`freya-dev`](skills/freya-dev/) | Build Rust desktop GUIs with Freya 0.5 (Skia-rendered, declarative) — scaffolding a working Cargo.toml, run/debug/hot reload, layout and styling, hooks and state, events, animation, routing, and diagnosing real linker/Skia/toolchain failures. | `npx skills add Natane0808/skills --skill freya-dev` |

## Layout

```
skills/
└── <skill-name>/
    ├── SKILL.md        # entrypoint: YAML frontmatter (name, description) + instructions
    ├── references/     # deep-dive docs loaded only when needed
    └── scripts/        # optional executables used by the skill
```

`SKILL.md` frontmatter `description` is what drives skill triggering, so it states both
what the skill does and when to use it.

## Installing a skill

### Via the skills CLI (recommended)

```bash
# Install one skill into the current project
npx skills add Natane0808/skills --skill nikala

# Or install every skill in this repo
npx skills add Natane0808/skills --all

# Install globally, available across all projects
npx skills add Natane0808/skills --skill nikala -g
```

The CLI detects which agents you have installed (Claude Code, Cursor, Codex, Windsurf,
and many more) and writes the skill into each agent's skills directory.

### Manually

Copy the skill directory into your agent's skills folder:

```bash
cp -r skills/nikala ~/.agents/skills/nikala
```

## Contributing

Each skill is self-contained: add `skills/<name>/SKILL.md` with `name` and `description`
frontmatter, plus optional `references/` and `scripts/`. Discovery walks every
`skills/**` container up to three levels deep, so a flat layout
(`skills/<name>/SKILL.md`) works as well as a categorized one
(`skills/<category>/<name>/SKILL.md`).

## License

MIT — see [LICENSE](LICENSE).
