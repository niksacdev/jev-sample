# ADR 0004: Consolidate engineering guidance

Date: 2026-10-05
Status: Accepted
Supersedes: [ADR 0003](0003-separate-principles-from-tools.md)

## Context

Separate Rust practices, AI practices, and tooling options duplicated standards
and increased context load and drift. The user approved removing them.

## Decision

Keep all mandatory engineering principles in root engineering-standards.md.
Keep AGENTS.md short: require the current standard and relevant ADRs, then load
only task-specific product or evaluation context. Copilot instructions remain
a thin discovery bridge, not a second rule source.

ADRs preserve significant decisions. Verified runbooks and executable configuration
describe actual commands and controls when implemented. Agent profiles and skills
contain concrete procedures only when needed, referencing the canonical standard.
Do not maintain a permanent catalog of provisional tools.

## Alternatives

Multiple explanatory practices documents offer categorization but duplicate
rules and increase retrieval overhead. One giant document mixing principles,
procedures, versions, and decisions becomes stale.

## Consequences

Preserve unique Rust principles and reference links in the standard, and the
independent review contract in ADR 0002. Tool choice remains an architecture task;
earlier candidate frameworks are not approved dependencies.
Product scope, model capabilities, metrics, and evaluation design remain distinct.

## Verification and follow-up

The three overlapping documents were removed and live references updated.
Historical statements in superseded ADRs retain their rationale.
No Cargo project, agent profile, skills, hooks, runbook, or CI exists yet.
Verify client discovery and implement controls in their respective milestones.
