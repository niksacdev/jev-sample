# ADR 0005: Maintain instructions through an evidence-driven review loop

Date: 2026-10-05
Status: Accepted

## Context

Versioning tracks edits but does not prevent repeated mistakes or stale guidance.
The user requested a local engineering-maintainer and approved human-reviewed
proposals based on repository findings and supplied session evidence.

## Decision

Use the language-agnostic pre-commit manager and YAML configuration for staged checks and a
fresh-context learning skill. Retain a read/search-only engineering-maintainer
profile for richer PR-review packets. Both assess evidence against existing instructions/checks
and proposes scoped corrections or returns no change. It cannot edit or adopt.
The harness applies human-accepted proposals as separately reviewable changes.
Prefer regression tests/checks where possible; do not expand root context for
every lesson. Add scoped AGENTS.md only where actual code needs specific rules.

## Alternatives

Automatic post-commit editing silently changes reviewed candidates and can
promote untrusted session content into authority. Hosted PR automation adds
permissions and cost before the local contract is validated; defer it.
Pre-committer alone validates code but does not maintain guidance.

## Consequences

Packets need candidate identity, bounded scope, and provenance. Client discovery
and tool restrictions must be verified. No external session harvesting or
automatic policy adoption. Updated obligations bump standard/AGENTS versions.

## Verification and follow-up

Update 2026-10-05: user chose automatic generation of separate proposed patches,
with human adoption. The Python staged-snapshot runner and its tests were rejected
and removed: custom implementation must be Rust. Automatic hook integration is
pending. The profile contract passed a manually instructed separate-context
synthetic smoke review; native discovery and live CLI behavior remain unverified.
Verify the eventual Rust implementation and usefulness on a real change.
