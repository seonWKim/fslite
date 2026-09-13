# Milestones

Implementation order for fslite, following the design in `README.md`. Each
milestone builds on the previous one and should be independently testable.

## Phase 1 — Single Node

### M1: Metadata layer
- Schema: `Project`, `EntityType`, `Feature` tables (embedded SQLite-compatible DB).
- `Feature` row stores its build params: sparse index block size (default
  4096), time window size (default 1 day).
- CRUD for registering/looking up projects, entity types, features.
- **Done when**: a project/entity_type/feature can be registered and resolved
  back to its params via the metadata layer, with no other component involved.

### M2: WAL record format
- Length-prefixed record encoding: `[len][entity_id][timestamp][value]`.
- Reader/writer utilities: append a record, sequentially iterate all records.
- **Done when**: records round-trip (write N records, read them back in order)
  for variable-length `entity_id`/`value`.

### M3: Write path
- One WAL per `(project, entity_type, feature)`.
- In-memory index: latest `(timestamp, value)` per `(entity_id, feature)`,
  updated synchronously on every write.
- **Done when**: a write is durably appended to the right WAL and immediately
  visible via the in-memory index.

### M4: Ingestion API
- Client connection scoped to `(project, entity_type, entity_id)`, streaming
  `(feature, timestamp, value)` tuples.
- Out-of-order timestamps accepted (no ordering assumption enforced).
- **Done when**: a client can open a connection and stream writes for an
  entity end-to-end into M3's write path.

### M5: Online read path
- Lookup: `project + entity_type + entity_id(s) + feature(s)`, no timestamp →
  latest value from the in-memory index.
- Multi-entity lookups fan out per-entity and merge results.
- **Done when**: a value written via M4 is readable through this API — closes
  the first full write→read loop.

### M6: Time-windowed compactor
- Background job per feature, fixed interval (default: 1 day window).
- Merges WAL entries into the current window's base file segment, sorted by
  `(entity_id, timestamp)`; atomic rewrite (write new file, rename over old).
- Closed windows are never touched again.
- **Done when**: WAL entries end up correctly merged into a sorted, per-window
  base file, repeatable across multiple compaction cycles.

### M7: Sparse index + offline read path
- Sparse index built per window segment during compaction: every ~4096 bytes,
  record `(next entity_id, byte offset)`; loaded fully into memory on file open.
- Lookup: binary-search the index → seek → linear scan (via length prefixes)
  until match, a larger `entity_id` (miss), or the next block.
- Read API: `timestamp t` + per-feature tolerance `y` → most recent value in
  `(t - y, t]`, always served from disk. No match → null.
- **Done when**: an as-of query against a compacted feature returns the
  correct value (or null), including across multiple windows.

### M8: Crash recovery
- On restart: reload each feature's base file(s), replay its WAL since the
  last compaction to rebuild the in-memory index.
- **Done when**: killing the process mid-write and restarting reproduces the
  same in-memory state as before the crash (modulo the known compaction-lag
  limitation).

### M9: Shard-resolution stub
- Every request (ingest + lookup) passes through a "resolve target node for
  this shard" step, even though phase 1 always resolves to the local node.
- **Done when**: this step exists in the request path and phase 2 can replace
  its implementation without touching the protocol, file layout, or metadata
  schema.

### M10: Source/Sink examples
- At least one Source adapter (e.g. reading a CSV or a Kafka topic) driving
  the ingestion API, and one Sink adapter (e.g. dumping offline reads to a
  training-set file) consuming the read API.
- **Done when**: an end-to-end example runs Source → fslite → Sink without
  manual glue code.

## Phase 2 — Horizontal Scaling (deferred)

- Single authoritative metadata/routing node owning the shard map.
- Sharding by `(project, entity_type, feature)`, and by `entity_id` range
  within a feature if needed.
- No cloud dependency (self-hosted; Turso Cloud-based replication ruled out).

## Known limitations carried from Phase 1

- **Compaction lag**: as-of reads only see the compacted base file, not the
  WAL tail — a timestamp inside the current compaction window can return
  stale/missing data until the next compaction runs.
