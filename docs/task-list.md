# Milestone task list

Updated: 2026-10-05. Pause for discussion after each logical milestone.
Statuses describe decisions and work completed, not just document creation.

| Milestone | Status | Outcome |
| --- | --- | --- |
| Verify Jev capabilities | Complete | Official documentation researched; no live inference performed |
| Select business outcome | Complete | Synthetic auto-insurance claim-intake triage approved |
| Define evaluation and product metrics | Complete | Hierarchy and initial scorecard targets approved |
| Design fair comparison | Complete | Design approved; execution details remain to be resolved |
| Define Rust implementation direction | Complete | Rust frameworks and practices; no Python comparison or superiority claim |
| Agree architecture | Pending | Select a small typed backend and adapters; record framework choices in ADRs |
| Establish engineering standards | In progress | Mandatory ES rules, ADR policy, and harness entrypoints exist; client loading and future check enforcement remain to verify |
| Set up pre-committer | Pending | Deterministic local/CI gates, bounded review agent, and justified reusable skills; hooks not yet selected |
| Build first Rust learning slice | Pending | Minimal API, focused tests, explanations, runnable commands; pause for review |
| Integrate providers incrementally | Pending | Fixtures/rules first, then Jev and structured LLM with explicit failure handling |
| Evaluate and document | Pending | Execute approved experiment; report evidence, limitations, and reproducible commands |

## Prerequisites

- Capability verification precedes outcome selection.
- Outcome selection precedes metric definition.
- Metric definition precedes comparison design.
- Comparison design and the Rust implementation direction precede architecture.
- Initial engineering standards and ADRs begin before architecture.
- Approved architecture precedes implementation-specific check configuration.
- Engineering standards precede pre-committer setup.
- Quality gates precede the first learning slice.
- The first slice precedes provider integration.
- Integration precedes evaluation.

## Current pause point

The comparison design and Rust-first direction are approved. Next, agree on
architecture before starting implementation.
