# Loon agent workflow

## webOS TV knowledge (required for `client/` work)

Before editing the webOS client, call MCP **`search_knowledge_base`** with
`collection="webos-tv"` for the platform APIs you need (appinfo, back button,
events, packaging, Developer Mode).

Index lives in the **nest** repo:

```bash
cd /path/to/nest && ./scripts/fetch-webos-knowledge.sh
```

See [docs/webos-knowledge-v1.md](docs/webos-knowledge-v1.md) and
[client/README.md](client/README.md).

## Context memory

Use `save_context_memory` with `session_key` = current git branch after each turn
when working from a nest workspace with hooks enabled.

## Client changes → commit and push (mandatory)

**Dev machine ≠ TV/simulator machine.** Any change under `client/` is untestable on
hardware until it is on `origin/main`.

After every client implementation or fix:

1. `npm test` and `npm run build` in `client/`
2. **Commit and push** to `pacificnm/loon` — do not leave fixes local-only
3. Tell the user the commit hash and TV reinstall steps (`package:webos` → `ares-package -n` → `ares-install`)

The user pulls on the TV/simulator box before testing. Skipping push blocks all
further verification.
