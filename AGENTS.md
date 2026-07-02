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
