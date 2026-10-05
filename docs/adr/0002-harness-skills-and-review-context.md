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

Update 2026-10-05: root AGENTS.md and Copilot repository instructions now exist
and reference the mandatory ES rules with selective supporting context.
Automatic loading in each client is not yet verified. No agent profile, skills,
hooks, or CI are configured. Validate context loading, isolated test execution, and failure
reporting during quality-gate setup.

Update 2026-10-05: supplementary practices documents were consolidated under
[ADR 0004](0004-consolidate-engineering-guidance.md). Preserve the review contract:
provide the exact candidate, acceptance criteria, standards, relevant ADRs, verified
commands, and known limitations. Review source evidence, not only the harness summary.
Report command results, finding locations/severity, decision deviations, and missing
checks. Test artifacts are permitted in the isolated surface; source edits are not.
Materialize staged candidates explicitly and rerun after changes.
The profile and isolation mechanism remain to be implemented and verified.

Clarification 2026-10-05: separate context is required for every subagent, not
unique to pre-committer. Its distinguishing role is independently inspecting
candidate evidence and executing checks, rather than merely repeating the
implementer's summary. Standards and harness instructions now carry a synchronized
version marker; reviews report the applied standard version.
