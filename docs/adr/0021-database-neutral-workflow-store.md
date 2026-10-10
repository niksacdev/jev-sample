# ADR 0021: Database-neutral workflow store

Date: 2026-10-10
Status: Accepted
Tracking issue: [22](https://github.com/niksacdev/zipclaim/issues/22)

## Context and user direction

Keep SQLite as ZipClaim's only backend today because it is easy to transport and
run locally. Plan for a later move to Postgres, but do not let SQLite become a
dependency of workflow services, HTTP handlers or the UI. The future Postgres
backend should be a new module plus migrations, without changes to those layers.

## Decision

`WorkflowStore` is a backend-neutral handle over a `WorkflowRepository` trait.
The trait exposes boxed, `Send` futures for admission, resume reservation,
request lookup, save, get and list. The SQLite implementation lives under
`store/sqlite.rs`; recovery after opening a repository is shared in `store/mod.rs`.
Existing callers continue to use `WorkflowStore::open(path, policy)` as the
SQLite convenience constructor, or can construct a handle with `WorkflowStore::new`.
Comparison identifiers remain external strings (`comparison-{n}`).

SQLite schema changes are versioned in `migrations/sqlite/`, loaded at compile
time and recorded in the portable `schema_migrations` table. Migrations are
transactional and unknown versions fail closed. SQL avoids SQLite-only insert
identity and quota expressions, uses explicit insert columns, and maintains
`snapshot_bytes` for quota accounting. A reusable repository contract test checks
behavior independently of a concrete backend.

## Postgres mapping notes

- Map `INTEGER id` to `BIGINT GENERATED ALWAYS AS IDENTITY`.
- Map `snapshot TEXT` to `JSONB`.
- Translate SQLite `?N` placeholders to Postgres `$N` placeholders.
- Replace SQLite page-count/PRAGMA quota configuration with the application-level
  `snapshot_bytes` quota.
- Replace local file permission checks with appropriately scoped DB roles and TLS.
- Put future scripts in `migrations/postgres/` and implement the same repository
  contract.
- A future `WorkflowStore::open` may dispatch based on a `DATABASE_URL` scheme;
  no URL dispatch is introduced now.

## Consequences and scope

Services and HTTP/UI code continue to depend on the same store methods. SQLite
remains the sole implementation and retains its local PRAGMA configuration,
permissions checks, append-only event checks, quota limits and interrupted-run
recovery. No Postgres code or Postgres migrations are included. Usage and cost
are not normalized into separate tables yet; snapshots remain the durable model.
