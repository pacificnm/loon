# webOS TV developer knowledge — plan

## Status: Implemented (fetch + index)

Index [LG webOS TV developer documentation](https://webostv.developer.lge.com/develop/references) into the **nest-knowledge** MCP stack.

---

## How to get the knowledge (run these commands)

From the **nest** repo root, with `.venv`, PostgreSQL, and `OPENAI_API_KEY` in `.env` (see [MCP-SETUP.md](../../../tools/MCP-SETUP.md)):

```bash
# 1. Install deps (once)
python3 -m venv .venv
.venv/bin/pip install -r tools/requirements.txt

# 2. Fetch all 8 LG pages → markdown, then embed into PostgreSQL
./scripts/fetch-webos-knowledge.sh

# 3. Verify search works
.venv/bin/python tools/search_knowledge.py "disableBackHistoryAPI" --collection webos-tv
```

**In Cursor:** reload MCP, then agents call:

```text
search_knowledge_base(query="appinfo.json mandatory fields", collection="webos-tv")
```

**Re-fetch** after LG doc updates:

```bash
./scripts/fetch-webos-knowledge.sh --force
```

### Where files land

| Location | Contents |
|----------|----------|
| `$NEST_KNOWLEDGE/webos-tv/*.md` | Cached markdown (default: `data/nest-knowledge/webos-tv/` in repo if `/data/nest-knowledge` not writable) |
| PostgreSQL `knowledge_base` | Embedded chunks, collection `webos-tv` |
| `tools/webos-knowledge-urls.toml` | The 8 source URLs |

Override output root:

```bash
NEST_KNOWLEDGE=/data/nest-knowledge ./scripts/fetch-webos-knowledge.sh
```

---

## Can we do this?

**Yes.** The knowledge indexer already supports arbitrary local markdown trees:

```text
Fetch LG pages → markdown files on disk → index_knowledge.py → PostgreSQL knowledge_base
                                                      ↑
                                            nest-knowledge MCP (semantic search)
```

It works the same way as `rust-book` and `egui` collections today (see [MCP-SETUP.md](../../../tools/MCP-SETUP.md)).

**What we do not need:** a new MCP server. Add a `webos-tv` collection to `tools/knowledge.toml` and extend the fetch/index scripts.

---

## Architecture

```text
https://webostv.developer.lge.com/develop/...
        │
        │  tools/fetch_webos_knowledge.py  (HTTP + HTML→markdown)
        ▼
/data/nest-knowledge/webos-tv/          # NEST_KNOWLEDGE root (outside git)
├── references/
│   ├── appinfo-json.md
│   ├── webos-events.md
│   ├── webostvjs-introduction.md
│   └── ...
├── guides/
│   ├── design-principles.md
│   ├── back-button.md
│   └── ...
└── manifest.json                       # source URL + fetched_at per file
        │
        │  ./scripts/index-knowledge.sh  (+ webos-tv collection)
        ▼
PostgreSQL knowledge_base (collection = webos-tv)
        │
        ▼
Cursor: search_knowledge_base("appinfo.json id field", collection="webos-tv")
```

### Why outside the repo?

Matches existing pattern: manuals live under `NEST_KNOWLEDGE` (default `/data/nest-knowledge`), not in git. LG owns the content; we store a **cached copy** with source URLs in `manifest.json` for refresh and attribution.

Optional: commit a **minimal** subset into `loon/docs/webos-lg/` for offline reading only — still index from `nest-knowledge` for search.

---

## Scope — canonical URL set (v1)

**These 8 pages are the complete v1 knowledge scope** (defined in [`tools/webos-knowledge-urls.toml`](../../../tools/webos-knowledge-urls.toml)):

| # | Topic | URL |
|---|-------|-----|
| 1 | appinfo.json | https://webostv.developer.lge.com/develop/references/appinfo-json |
| 2 | webOS Events | https://webostv.developer.lge.com/develop/references/webos-event |
| 3 | Luna Service intro | https://webostv.developer.lge.com/develop/references/luna-service-introduction |
| 4 | webOSTV.js intro | https://webostv.developer.lge.com/develop/references/webostvjs-introduction |
| 5 | webos-service intro | https://webostv.developer.lge.com/develop/references/webos-service-introduction |
| 6 | Developer workflow | https://webostv.developer.lge.com/develop/getting-started/developer-workflow |
| 7 | Web app types | https://webostv.developer.lge.com/develop/getting-started/web-app-types |
| 8 | Developer Mode app | https://webostv.developer.lge.com/develop/getting-started/developer-mode-app |

The fetch script pulls **only** URLs listed in the manifest — not the whole LG site. Child pages linked from Luna Service (Activity Manager, Application Manager, etc.) are **not** included unless added to the manifest.

### Optional P2 expansion (not in v1 scope)

Add later if needed: design principles, back button, webOS API reference, Magic Remote, mediaOption, app lifecycle guides.

---

## Implementation tasks

### 1. URL manifest (`tools/webos-knowledge-urls.toml`)

```toml
[[pages]]
slug = "references/appinfo-json"
url = "https://webostv.developer.lge.com/develop/references/appinfo-json"
priority = 0

[[pages]]
slug = "guides/design-principles"
url = "https://webostv.developer.lge.com/develop/guides/design-principles"
priority = 0
# ...
```

### 2. Fetch script (`tools/fetch_webos_knowledge.py`)

| Feature | Approach |
|---------|----------|
| HTTP | `httpx` or `urllib` with User-Agent `LoonDocMirror/1.0` |
| HTML → markdown | `markdownify` or `html2text` on main content region |
| Rate limit | 1 req/s, retry 429 with backoff |
| Output | `{NEST_KNOWLEDGE}/webos-tv/{slug}.md` |
| Frontmatter | YAML: `source_url`, `fetched_at`, `title` |
| Idempotent | Skip if URL unchanged and `--force` not set |

**Note:** Manual `WebFetch` in Cursor already returns clean markdown for single pages (verified on `appinfo.json`). The script automates that for the full set.

### 3. Shell wrapper (`scripts/fetch-webos-knowledge.sh`)

```bash
NEST_KNOWLEDGE="${NEST_KNOWLEDGE:-/data/nest-knowledge}" \
  .venv/bin/python tools/fetch_webos_knowledge.py \
  --config tools/webos-knowledge-urls.toml \
  --output "${NEST_KNOWLEDGE}/webos-tv"
```

### 4. Extend `scripts/index-knowledge.sh`

Append to generated `tools/knowledge.toml`:

```toml
[[collections]]
name = "webos-tv"
source = "${KNOWLEDGE}/webos-tv"
extensions = ["md"]
```

### 5. Dependencies

Add to `tools/requirements.txt` (if not present):

```text
httpx
markdownify
```

### 6. Agent workflow

When working on `loon/client/`:

1. `search_knowledge_base(query, collection="webos-tv")` — platform APIs, appinfo, back button
2. `search_project_memory` — Loon client plans in loon repo (after indexed separately or via context)
3. `search_context_memory` — session continuity

Optional AGENTS.md / loon client rule:

> When editing Loon webOS client code, search `webos-tv` knowledge for platform APIs before guessing.

### 7. Refresh cadence

Re-run fetch + index when:

- Starting a new client phase (W0, W1, …)
- LG docs linked from release notes change
- Search returns stale/conflicting answers

```bash
./scripts/fetch-webos-knowledge.sh
./scripts/index-knowledge.sh   # re-embeds changed chunks (ON CONFLICT DO NOTHING on hash)
```

---

## Verification

```bash
# After fetch
ls /data/nest-knowledge/webos-tv/references/

# After index
.venv/bin/python tools/search_knowledge.py "disableBackHistoryAPI" --collection webos-tv
.venv/bin/python tools/search_knowledge.py "Magic Remote key codes" --collection webos-tv

# MCP
# search_knowledge_base(query="appinfo.json mandatory fields", collection="webos-tv")
```

---

## Legal & practical notes

| Topic | Guidance |
|-------|----------|
| **Terms** | LG developer site content is for building webOS apps; mirror for local dev search, not republication. Keep `source_url` in every file. |
| **Robots** | Fetch politely (low rate, identified User-Agent). No login-gated pages. |
| **Stability** | LG may redesign URLs; manifest + slug paths make diffs visible on re-fetch. |
| **Embeddings cost** | ~80 pages × ~10 chunks ≈ 800 embeddings — one-time small OpenAI cost per full re-index. |
| **Offline** | Fetched markdown works without network after first pull. |

---

## Alternatives considered

| Approach | Pros | Cons |
|----------|------|------|
| **nest-knowledge collection** (chosen) | Same MCP, semantic search, proven | Needs fetch script + NEST_KNOWLEDGE dir |
| **project_memory** (nest `docs/`) | Already indexed | LG content not Nest-owned; loon not in nest index paths |
| **Commit md into loon repo** | Simple git clone | Large diffs, license noise, no semantic search without index anyway |
| **Live WebFetch per agent turn** | Always fresh | Slow, rate limits, inconsistent in hooks |

---

## Nest vs Loon repo ownership

| Artifact | Repo |
|----------|------|
| `fetch_webos_knowledge.py`, URL manifest, index-knowledge change | **nest** (`tools/`, `scripts/`) |
| This plan, client links | **loon** (`docs/webos-knowledge-v1.md`) |
| Fetched markdown files | **neither** — `NEST_KNOWLEDGE/webos-tv/` |

---

## Related

- [MCP-SETUP.md](../../../tools/MCP-SETUP.md) — knowledge base setup
- [knowledge.toml.example](../../../tools/knowledge.toml.example)
- [client/README.md](../client/README.md) — Loon UI plan
- [LG References](https://webostv.developer.lge.com/develop/references)
