# ADR 0001: Bound the sample to insurance intake

Date: 2026-10-05
Status: Accepted

## Context

The sample must distinguish Jev's narrow typed judgments from LLM generation and
exact computation. Financial forecasting and consequential eligibility decisions
are not justified by the verified capabilities.

## Decision

Use synthetic English auto-insurance narratives for incident classification,
urgent-assistance cues, and explicit human-review routing. Rust owns validation
and routing. Compare Jev with structured-output LLM and deterministic baselines.
Exclude coverage, liability, fraud accusations, payouts, and emergency automation.

## Alternatives

Loan-servicing triage and investment-document classification were plausible.
Stock forecasting and autonomous lending were not selected.

## Consequences

Results can support the bounded sample, not production certification. Business
savings require a user exercise, not synthetic accuracy alone.

## Verification and follow-up

Scope, metric hierarchy/targets, and evaluation design are user-approved; see
[product scope](../product-scope.md), [metrics](../metrics.md), and
[evaluation design](../evaluation-design.md). No model evaluation or backend
implementation has occurred. Framework selection remains an architecture task.

Clarification 2026-10-05: this records the initial approved scope, not an immutable
feature restriction. New user requirements may evolve the product. Update product
documents and supersede this ADR when a consequential scope decision replaces it,
following the standard's
[change discipline](../../engineering-standards.md#change-discipline)
rather than requiring redundant approval of clear intent.
