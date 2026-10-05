# Milestone task list

Updated: 2026-10-05. Pause for discussion after each logical milestone.
Statuses describe decisions and work completed, not just document creation.

| Milestone | Status | Outcome |
| --- | --- | --- |
| Verify Jev capabilities | Complete | Official documentation researched; no live inference performed |
| Select business outcome | Complete | Synthetic auto-insurance claim-intake triage approved |
| Define evaluation and product metrics | Complete | Hierarchy and initial scorecard targets approved |
| Design fair comparison | In progress | Design proposed; awaiting approval |
| Define Rust quality hypothesis | Pending | Separate compiler guarantees from architecture/testing quality; define measurable criteria |
| Agree architecture | Pending | Small typed backend and adapters; Axum/Tokio tentative |
| Write coding standards | Pending | Create coding-standards.md for domain, design, errors, precision, tests, secrets, dependencies |
| Set up pre-committer | Pending | Deterministic local/CI gates plus standards-aware agent; hook mechanism not yet selected |
| Build first Rust learning slice | Pending | Minimal API, focused tests, explanations, runnable commands; pause for review |
| Integrate providers incrementally | Pending | Fixtures/rules first, then Jev and structured LLM with explicit failure handling |
| Evaluate and document | Pending | Execute approved experiment; report evidence, limitations, and reproducible commands |

## Prerequisites

- Capability verification precedes outcome selection.
- Outcome selection precedes metric definition.
- Metric definition precedes comparison design.
- Comparison design and the Rust hypothesis precede architecture.
- Architecture precedes coding standards.
- Coding standards precede pre-committer setup.
- Quality gates precede the first learning slice.
- The first slice precedes provider integration.
- Integration precedes evaluation.

## Current pause point

The comparison design is proposed, not approved. Discuss it before starting
dataset generation or implementation.
