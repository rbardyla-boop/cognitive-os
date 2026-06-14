# Backend

## Storage

v0.1 uses local SQLite. The initial migration creates:

- `packets`
- `episodes`
- `memory_nodes`
- `rules`
- `procedures`
- `contradictions`
- `traces`
- `deferred_jobs`
- `system_events`

Packet rows store `schema_version` and `raw_json` so older packet logs remain readable after later schema updates.

## API

The local stdlib HTTP API exposes:

- `GET /health`
- `POST /input`
- `GET /packets`
- `GET /traces/:id`
- `GET /memory/:id`
- `GET /system-state`
- `POST /simulate/scenario`

Run locally:

```sh
python3 scripts/backend_api.py 8765
```

## Migrations

Every schema update is represented as a migration in `scripts/backend_storage.py`. Applied migrations are tracked in `schema_migrations`.

