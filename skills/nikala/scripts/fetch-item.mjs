#!/usr/bin/env node
/**
 * fetch-item.mjs — inspect a Nikala UI registry item without the CLI.
 *
 * Use this when:
 *   - the project has no `@nikala-ui/cli` installed,
 *   - you need the exact source the registry would install,
 *   - or the MCP server is not configured.
 *
 * Usage:
 *   node fetch-item.mjs list                 # full registry catalogue
 *   node fetch-item.mjs button               # show item metadata + file paths
 *   node fetch-item.mjs button --source      # also print file contents
 *   node fetch-item.mjs --search dialog      # find items whose name/description matches
 *   node fetch-item.mjs list --json          # raw JSON
 *
 * Exit codes: 0 ok, 1 usage error, 2 network/parse error, 3 item not found.
 */

/**
 * Registry mirrors, tried in order. The official GitHub raw host is canonical, but it is
 * blocked on some networks (ECONNRESET behind certain firewalls/CI egress rules), so a
 * CDN mirror of the same `main` branch is kept as a fallback. Override with
 * NIKALA_REGISTRY_BASE to point at a private registry.
 */
const REGISTRY_BASES = [
  process.env.NIKALA_REGISTRY_BASE,
  "https://raw.githubusercontent.com/nikala-ui/ui/main/packages/core/registry",
  "https://cdn.jsdelivr.net/gh/nikala-ui/ui@main/packages/core/registry",
].filter(Boolean);

const REGISTRY_BASE = REGISTRY_BASES[0];

const args = process.argv.slice(2);

function usage(code = 1) {
  console.error(
    [
      "fetch-item.mjs — Nikala UI registry inspector",
      "",
      "  node fetch-item.mjs list [--json]",
      "  node fetch-item.mjs <item-name> [--source]",
      "  node fetch-item.mjs --search <query>",
      "",
      "Registry: " + REGISTRY_BASE,
    ].join("\n")
  );
  process.exit(code);
}

if (args.length === 0 || args.includes("--help") || args.includes("-h")) usage(args.length ? 0 : 1);

const wantsJson = args.includes("--json");
const wantsSource = args.includes("--source");
const searchIdx = args.indexOf("--search");

async function getJson(url) {
  const cacheBust = url.includes("?") ? "&" : "?";
  const res = await fetch(`${url}${cacheBust}t=${Date.now()}`, {
    headers: {
      "User-Agent": "nikala-skill-fetch-item/1.0",
      "Cache-Control": "no-cache, no-store",
    },
  });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`);
  return res.json();
}

/**
 * Fetch a registry path from the first mirror that answers. `null` means every mirror
 * reported 404 (the item genuinely does not exist); a throw means no mirror was reachable.
 */
async function getRegistryJson(relPath) {
  const failures = [];
  for (const base of REGISTRY_BASES) {
    try {
      return await getJson(`${base}/${relPath}`);
    } catch (err) {
      failures.push(`${base}: ${err.message}`);
    }
  }
  throw new Error(`no registry mirror reachable\n  ${failures.join("\n  ")}`);
}

/** Human-readable one-line summary: "button  [ui]  An interactive button…" */
function summarise(item) {
  const deps = item.registryDependencies?.length
    ? `  ↳ registry: ${item.registryDependencies.join(", ")}`
    : "";
  const npm = item.dependencies?.length ? `  ↳ npm: ${item.dependencies.join(", ")}` : "";
  return `${item.name}\t[${item.type.replace("registry:", "")}]\t${item.description ?? ""}${deps}${npm}`;
}

async function main() {
  // --- search mode -------------------------------------------------------
  if (searchIdx !== -1) {
    const query = (args[searchIdx + 1] ?? "").toLowerCase();
    if (!query) usage();
    const index = await getRegistryJson("index.json");
    if (!index) throw new Error("registry index unavailable");
    const hits = index.filter(
      (i) =>
        i.name.toLowerCase().includes(query) ||
        (i.title ?? "").toLowerCase().includes(query) ||
        (i.description ?? "").toLowerCase().includes(query)
    );
    if (hits.length === 0) {
      console.error(`No items matched "${query}".`);
      process.exit(3);
    }
    if (wantsJson) console.log(JSON.stringify(hits, null, 2));
    else hits.forEach((h) => console.log(summarise(h)));
    return;
  }

  const target = args.find((a) => !a.startsWith("-"));

  // --- list mode ---------------------------------------------------------
  if (!target || target === "list") {
    const index = await getRegistryJson("index.json");
    if (!index) throw new Error("registry index unavailable");
    if (wantsJson) {
      console.log(JSON.stringify(index, null, 2));
      return;
    }
    const groups = new Map();
    for (const item of index) {
      const key = item.type.replace("registry:", "");
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key).push(item);
    }
    for (const [type, items] of [...groups.entries()].sort()) {
      console.log(`\n# ${type} (${items.length})`);
      for (const item of items.sort((a, b) => a.name.localeCompare(b.name))) {
        console.log(`  ${item.name}\t${item.description ?? ""}`);
      }
    }
    console.log(`\nTotal: ${index.length} items`);
    return;
  }

  // --- single item -------------------------------------------------------
  const item = await getRegistryJson(`${target}.json`);
  if (!item) {
    console.error(`Item "${target}" not found in the registry.`);
    console.error(`Try: node fetch-item.mjs --search ${target}`);
    process.exit(3);
  }
  if (wantsJson) {
    console.log(JSON.stringify(item, null, 2));
    return;
  }

  console.log(`${item.title ?? item.name}  (${item.name})`);
  console.log(`type:        ${item.type}`);
  if (item.description) console.log(`description: ${item.description}`);
  if (item.dependencies?.length) console.log(`npm deps:    ${item.dependencies.join(", ")}`);
  if (item.registryDependencies?.length)
    console.log(`registry deps: ${item.registryDependencies.join(", ")}  (installed automatically)`);

  console.log(`\nfiles (${item.files.length}):`);
  for (const f of item.files) {
    console.log(`  - ${f.path}  (${f.type})`);
  }

  if (wantsSource) {
    for (const f of item.files) {
      console.log(`\n${"=".repeat(70)}\n${f.path}\n${"=".repeat(70)}`);
      console.log(f.content);
    }
  } else {
    console.log(`\nRe-run with --source to print file contents.`);
  }
}

main().catch((err) => {
  console.error(`error: ${err.message}`);
  process.exit(2);
});
