# fslite

The simplest feature store implementation, written in Rust. fs (feature store) + lite.

## Problem & Purpose

Existing feature stores (e.g., Feast) are heavy — separate registry/online/offline
services with a big dependency footprint (Redis, Spark, Postgres, etc.), costly to
run and integrate. fslite: the lightest, fastest feature store, usable as an
embedded library or a standalone server via simple API calls.

- **Top priority**: online point-lookup latency.
- **Secondary priority**: efficient offline (point-in-time) reads from disk, for
  training-set generation.

## Components

- **Project**: unit of access control / namespace, e.g., `fraud-detection`.
- **Entity type**: category features are grouped under, e.g., `user`, `stock`. Each
  instance (an *entity*) has an `entity_id`, e.g., `user#42`.
- **Feature**: a value describing an entity, e.g., `user.age`, `stock.price`.
- **Source**: where data is ingested from (Kafka, CSV, a DB table, ...).
- **Sink**: where data is sent to (training pipeline, online store, dashboard, ...).

## Architecture

- TODO: visual diagram of `Project -> Entity Type -> Entity -> Feature`.

**Metadata layer** — Projects, entity types, and features live in SQLite (or
Turso): embedded, reliable, no separate service.

**Ingestion** — A client connection is scoped to one `(project, entity_type,
entity_id)` and streams `(feature, timestamp, value)`. Timestamps may arrive
out of order (backfills supported).

**File layout (per feature)** — Each `(project, entity_type, feature)` has a WAL
and a base file, both using length-prefixed records `[len][entity_id][timestamp]
[value]` (entity_id/value are variable-length, so no fixed-record indexing, no
separate value file, no B+Tree):

- *WAL*: append-only, arrival order, sequential-only access (replay / compaction).
- *Base file*: all entities for that feature, sorted by `(entity_id, timestamp)`.
  Fully rebuilt each compaction (merge WAL in, write new file, atomic rename) —
  never mutated in place.
- *Sparse index*: built during compaction — every ~4KB written, record
  `(next entity_id, byte offset)`. Byte-based so worst-case scan per lookup is
  bounded despite variable record sizes. Small enough to load fully into memory.
- *Lookup*: binary-search the in-memory index → seek to block → linear scan
  (via length prefixes) until match, a larger entity_id (miss), or next block.

**Write path** — Write appends to the WAL and synchronously updates an in-memory
index of the **latest** `(timestamp, value)` per `(entity_id, feature)` only (no
history). A per-feature background compactor runs on a fixed interval, doing the
WAL→base-file merge above.

**Read path**

- *Online* (no timestamp): latest value from the in-memory index — O(1), fastest path.
- *Offline* (timestamp `t` + per-feature tolerance `y`): always disk, via the base
  file lookup above, most recent value in `(t - y, t]`. No match → null.
- Multi-entity lookups fan out per-entity and merge results.

**Crash recovery** — Reload base file + replay WAL since last compaction to
rebuild the in-memory index.

## Known limitations (Phase 1)

- **Compaction lag**: point-in-time reads only see the compacted base file, not
  the WAL tail — a timestamp inside the current compaction window may return
  stale/missing data until the next compaction. Accepted trade-off for keeping
  the online path simple (latest-only, no bounded history in memory).
