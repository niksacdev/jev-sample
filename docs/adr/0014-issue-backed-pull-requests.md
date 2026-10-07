# ADR 0014: Track work in GitHub issues and link every PR

Date: 2026-10-06
Status: Accepted
Tracking issue: [#6](https://github.com/niksacdev/jev-sample/issues/6)

## Context

The user identified the Markdown milestone task list as stale and requested
a GitHub issue for every PR going forward. Keeping a separate active task list
allows progress and scope to drift from implementation and review.

## Decision

GitHub issues are the source of truth for active tasks. Engineering standard
0.5.0 requires an issue before PR-bound implementation and explicit issue linkage
in every PR body, including documentation, maintenance, and policy changes.
Use closing keywords only when the issue's acceptance criteria are complete.
Partial delivery references the issue and leaves remaining work tracked there.

Mark docs/task-list.md as historical rather than deleting its decision evidence.
Do not migrate its unfinished entries automatically; review their current
relevance and scope first. Previously merged PRs need no retrospective issues.

## Alternatives

Maintaining both an active Markdown checklist and issues duplicates status and
recreates the observed drift. Requiring issues only for features leaves fixes
and engineering changes without consistent acceptance criteria.

## Consequences

The root standard remains the canonical rule source. AGENTS.md points agents
to the issue workflow, and a PR template prompts authors for issue linkage,
acceptance evidence, and limitations. Review verifies the requirement.
This change adds no automated linkage check or branch protection and grants
no additional authority to commit, push, merge, or deploy.

## Verification and follow-up

This policy change has a tracking issue. Check that the standard and AGENTS.md
version markers match, the PR template requires a link, and the README directs
active work to issues. Assess future PRs against their linked acceptance criteria.
