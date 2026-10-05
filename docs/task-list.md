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
| Agree architecture | Complete | [Approved single-package architecture](adr/0007-single-package-api-and-evaluation.md): shared library, Axum/Tokio HTTP server, evaluation CLI |
| Establish engineering standards | Complete | Standard 0.4.0 and linked ADR/harness guidance established; native discovery and automatic learning remain limitations |
| Set up pre-committer | Complete | Local hooks and hosted CI verified; isolated explicit-profile review exercised; native discovery and branch protection remain unconfigured |
| Build first Rust learning slice | Complete | Health API and four Axum/Tower tests; real HTTP, Git hooks, isolated review, and hosted CI passed for b47cb77 |
| Integrate providers incrementally | Pending | Fixtures/rules first, then Jev and structured LLM with wiremock HTTP contract/failure tests |
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

Architecture is approved. Quality-gate configuration is recorded in
[ADR 0009](adr/0009-ci-and-independent-candidate-review.md).
The local health API milestone is complete. Hosted run 37386081726 passed for
b47cb77. Independent review verified tree 6f1ad2df3c857640df02e80e16a3454242b494c0
under standard 0.4.0; its stale ADR finding was corrected before acceptance.
A deliberate wrong JSON response failed the contract test and was restored.
The real HTTP checks returned 200/404/405 and duplicate binding failed visibly.
Pause before typed claim/routing implementation. No provider behavior exists.
