# ADR 0002: Separate implementation guidance from independent review

Date: 2026-10-05
Status: Accepted

## Context

The user requires efficient AI-native development, engineering standards, fresh
decision memory, and an independently contextualized pre-committer that tests
and reviews. A generic implementation agent would duplicate the harness.

## Decision

The harness implements and integrates. Canonical engineering-standards.md and
relevant ADRs supply durable guidance through concise agent instructions.
Skills supply reusable task procedures in the harness's existing context.

Pre-committer has separate context, runs approved checks on an isolated exact
candidate snapshot, and reviews standards and architecture. It does not modify
source, commit, push, or automatically accept changes.

Security/performance/research are workflow responsibilities. Use additional
specialists only when substantial targeted work warrants them. Defer hosted
GitHub Agentic Workflows until local review is validated and permissions,
outputs, triggers, and budgets are agreed.

## Alternatives

An implementation-owner subagent duplicates responsibility. Many standing agents
add latency, cost, and integration complexity. Skills alone do not provide
independent review context. Hosted agentic workflows orchestrate AI execution;
they do not replace deterministic CI or human approval.

## Consequences

Keep instructions concise and procedures reusable. Test code executes with
bounded privileges and no production secrets. Review findings must identify
the checked revision, evidence, and missing checks. Rerun after candidate changes.
ADRs stay fresh via dated evidence or explicit supersession, not erased rationale.

## Verification and follow-up

No agent profile, skills, instruction entrypoints, hooks, or CI are configured
yet. Validate actual context loading, isolated test execution, and failure
reporting during quality-gate setup. See
[engineering practices](../ai-engineering-practices.md).
