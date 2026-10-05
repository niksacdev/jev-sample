# Rust engineering principles

Status: durable guidance for the approved Rust-first direction.
This document complements the [engineering constitution](../engineering-standards.md).
It does not mandate frameworks, packages, editor extensions, or command syntax.

## Principles

| Principle | Expectation |
| --- | --- |
| Make invalid states difficult to represent | Model domain concepts with meaningful types, explicit variants, and validated construction |
| Make ownership and lifetimes intentional | Prefer clear borrowing and ownership; introduce shared mutation only when justified |
| Keep contracts explicit | Separate domain behavior from transport and provider representations; document invariants and failure semantics |
| Validate at trust boundaries | Deserialization is not validation; check completeness, allowed values, numerical bounds, and consistency |
| Handle uncertainty and failure honestly | Distinguish domain uncertainty from technical failure; never substitute success-shaped defaults |
| Favor safe, maintainable code | Prefer safe abstractions and readable control flow; exceptional low-level code needs documented invariants and review |
| Bound resource use | Design for deadlines, cancellation, backpressure, and bounded work; avoid hidden blocking or uncontrolled concurrency |
| Prove behavior, not just compilation | Test acceptance criteria, invariants, boundary conditions, failure paths, and public contracts |
| Keep verification reproducible | Validate the exact candidate with repeatable builds and checks; report missing evidence explicitly |
| Measure before optimizing | Use representative measurements; assess correctness and maintainability alongside speed and resource cost |
| Minimize dependency and abstraction debt | Add only justified dependencies and layers; evaluate maintenance, compatibility, security, and licensing |
| Design for evolution | Preserve intentional contracts, document compatibility changes, and record consequential decisions in ADRs |

The compiler supplies important safety guarantees, not proof of sound architecture,
adequate tests, correct insurance logic, or reliable model judgments. Prefer
composition and focused modules over speculative generality. Neither raw coverage
nor code volume establishes quality.

## Established references

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):
  idiomatic, predictable, documented APIs with deliberate type and error design.
  Apply relevant recommendations; this is not a blanket mandate for every checklist item.
- [The Rust Programming Language](https://doc.rust-lang.org/book/):
  ownership, enums and pattern matching, error handling, and testing fundamentals.
- [Rust Reference](https://doc.rust-lang.org/reference/):
  authoritative language semantics when correctness depends on exact behavior.

References inform judgment; they do not override approved project boundaries.
Review their relevance as the language evolves rather than copying entire external
checklists into permanent standards.

## Principles versus implementation

Use [ADRs](adr/README.md) for accepted consequential choices and
[tooling options](tooling-options.md) for provisional candidates. Once configured,
actual manifests, scripts, and CI plus a verified runbook define current commands.
No proposal should be presented as an active safeguard.

A replacement tool must still satisfy the underlying purpose: reproducibility,
validation, behavioral evidence, security, or maintainability. Update affected
ADRs and runbooks without rewriting enduring principles to name each new tool.

Deliver small learning milestones with explanations and runnable tests. Keep
the harness, skills, and independent review boundaries described in the
[AI-native workflow](ai-engineering-practices.md).
