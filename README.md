# Skills

A collection of agent skills — reusable, task-specific instruction packages that an AI
coding agent loads when a matching task appears.

Each skill lives in its own directory under [`skills/`](skills/) and follows the
`SKILL.md` + `references/` + `scripts/` structure.

## Available skills

| Skill | Description |
| :--- | :--- |
| [`nikala`](skills/nikala/) | Build SolidJS + Tailwind CSS v4 + Tauri v2 interfaces with the [Nikala UI](https://nikala.dev) copy-paste component system — CLI workflow, verified component/hook catalog, design tokens, desktop suite, and troubleshooting. |

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

Copy the skill directory into your agent's skills folder, for example:

```bash
cp -r skills/nikala ~/.agents/skills/nikala
```

## License

MIT — see [LICENSE](LICENSE).
