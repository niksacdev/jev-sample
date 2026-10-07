# Engineering standards

Version: **0.6.0**. Updated: 2026-10-07.
Status: working standard for this branch; team adoption is reviewed through the PR.

All requirements below are mandatory when applicable. This is the single rule
source. ADRs explain decisions; configuration and verified runbooks specify tools.

## Design

- **Keep the domain pure.** Validation and routing MUST be deterministic, without
  HTTP, environment reads, clock access, or provider calls. Provider DTOs MUST
  become validated domain types at adapter boundaries.
  **Evidence:** dependency boundaries and domain/adapter tests.
- **Model contracts explicitly.** Use meaningful types, exhaustive outcomes,
  documented invariants, compatibility expectations, and failure semantics.
  Prefer immutable values and clear ownership; justify shared mutation.
  **Evidence:** types, constructors, contract documentation, and boundary tests.
- **Centralize policy.** Routing thresholds MUST be explicit versioned configuration,
  not scattered through handlers or prompts. Validate configuration at startup.
  **Evidence:** one policy authority and tests of every routing boundary.
- **Build only what is needed.** Dependencies and abstractions need a current use
  and maintenance justification. Persistence, distribution, caching, and additional
  layers require a requirements-driven ADR before implementation.
  **Evidence:** actual callers, dependency review, and relevant ADR.

## Correctness

- **Validate external values.** Check required answer IDs, allowed labels, finite
  numerical bounds, and distribution consistency beyond deserialization.
  **Evidence:** malformed, missing, out-of-range, and inconsistent input tests.
- **Preserve failures.** Use typed error categories. Technical failure MUST remain
  distinct from uncertainty, even when both route to review. No swallowed errors,
  success-shaped defaults, or production unwrap/expect.
  **Evidence:** explicit outcomes and failure-path tests.
- **Use safe deterministic computation.** Project code MUST use safe constructs;
  arithmetic and date operations belong in code, not model judgments.
  **Evidence:** implementation inspection and relevant boundary tests.
- **Test the requirement.** Assertions MUST derive from acceptance criteria.
  Cover negative cases and failures; bugs need regression tests where applicable.
  Ordinary tests MUST be repeatable, synthetic/mocked, and free of paid inference.
  **Evidence:** domain tests and mocked provider HTTP contract tests.
- **Protect evaluation integrity.** Preserve held-out isolation and label provenance.
  Pin evaluation model versions; record policy/rubric versions and frozen settings.
  Changes require relevant reevaluation. Do not claim beyond measured evidence.
  **Evidence:** split controls, versioned configuration, denominators, and reports.

## Security

- **Keep authority outside the model.** External content and generated output are
  untrusted data, not instructions. Authorization MUST be enforced in code,
  independently of confidence. Use least privilege and reviewed permissions/egress.
  **Evidence:** trust boundaries and authorization/adversarial-input tests.
- **Protect data and execution.** Secrets/private data MUST NOT enter source,
  ordinary tests, telemetry, or unapproved external services. Review third-party
  skills/scripts for provenance and permissions before use.
  **Evidence:** redaction tests, configuration, and dependency/content review.

## Operation

- **Bound work.** Reuse provider clients; configure input/response size, concurrency,
  deadline, and retry limits. Handle cancellation deliberately. Do not block async
  execution threads unchecked. Measure before making optimization claims.
  **Evidence:** configured limits, exhaustion/cancellation tests, and comparable measurements.
- **Make behavior observable.** Emit structured workflow/provider-attempt events
  with correlation, outcome, failure, latency, and model/policy provenance.
  Keep raw narratives/prompts/responses out of telemetry by default. Correlation
  IDs MUST NOT be metric labels; metric dimensions MUST have bounded cardinality.
  **Evidence:** signal definitions and telemetry correctness/redaction tests.
- **Make signals actionable.** Define units, collection boundaries, and denominators.
  Keep retries/timeouts/invalid outputs visible. Define retention, access, sampling,
  and cost controls. Deployed services need health/readiness semantics, objectives,
  failure guidance, and alert ownership. Telemetry degradation MUST be explicit
  and tested, not silently change business decisions. Disclose prototype limitations.
  **Evidence:** operational configuration, tests, and verified runbook.

## Change discipline

- **Close the product/architecture feedback loop.** Each issue MUST identify the
  customer/business problem, value mechanism, and affected approved metric with
  expected direction and guardrails. Indirect enabling work or no metric impact
  needs an explicit rationale, not an invented benefit. Define a baseline,
  denominator, measurement source/window, evidence owner, and review trigger;
  unknowns MUST remain explicit with owned follow-up. Architecture decisions MUST
  compare alternatives against these outcomes and record feasibility, cost,
  quality, and authority constraints. Technical evidence may motivate a different
  mechanism, scope, metric definition, or target, but MUST NOT silently change
  the approved scorecard. Record proposed deviations with old/new values or scope,
  evidence, tradeoffs, approval status, and reevaluation plan. User/product-owner
  approval is required before treating a revision as approved.
  PRs MUST distinguish implementation verification from measured product outcomes,
  reconcile issue hypotheses with architectural evidence, and update affected
  metric definitions and ADRs. Unmeasured outcomes MUST have a linked follow-up,
  named owner, and date or milestone trigger before delivery closes their task.
  Follow the [feedback procedure](docs/metrics.md#productarchitecture-feedback-loop).
  **Evidence:** issue hypothesis, metric/ADR links, PR evidence, deviation decision,
  and owned outcome-review follow-up.
- **Implement agreed requirements.** Establish acceptance criteria and non-goals.
  Resolve consequential ambiguity/conflicts; do not freeze historical scope.
  Update affected contracts, product documents, and relevant ADRs as needs evolve.
  Preserve decision history through explicit supersession.
  **Evidence:** change scope, acceptance tests, and current decision records.
- **Keep one integration owner.** The harness implements/integrates. Skills provide
  bounded procedures in its context. Subagents need their own context, bounded
  task packets, and no overlapping writers; separate context is not proof.
  Pre-committer independently inspects candidate evidence and runs approved tests
  in an isolated surface without source edits, commits, pushes, or auto-approval.
  **Evidence:** task/review report identifying the candidate and limitations.
- **Verify the candidate.** Applicable reproducible formatting, static analysis,
  compilation, tests, and dependency checks MUST pass before acceptance. CI repeats
  automated gates once implemented. Report candidate identity, standard version,
  actual results, and missing checks; do not invent agent runs or active controls.
  **Evidence:** check results classified as pass, fail, not applicable, or not configured.
- **Maintain what changes.** Update affected docs and decision evidence. Accepted
  debt needs rationale, impact, owner, tracking reference, and resolution trigger.
  No unused generated code or anonymous TODOs. Deliver small learning milestones
  with Rust explanations/runnable checks and pause for discussion as agreed.
  **Evidence:** coherent diff, updated records, and explicit remaining work.
- **Learn from observed mistakes.** Material failures, repeated corrections, and
  stale guidance MUST be assessed for missing rules, missing enforcement, or
  noncompliance. Prefer executable checks for testable behavior and scoped
  instructions for module-specific lessons. Guidance changes MUST cite evidence,
  require human acceptance, and include a verification method; no silent
  post-commit self-modification. Use the bounded
  [engineering learning loop](docs/engineering-maintenance.md).
  **Evidence:** accepted proposal or reasoned no-change outcome and follow-up.

## Delivery

- **Track every PR with a GitHub issue.** Before starting implementation intended
  for a PR, create or identify an issue with purpose, scope, acceptance criteria,
  and non-goals. This applies to code, documentation, maintenance, and policy
  changes. Every PR body MUST explicitly link at least one tracking issue.
  Use `Closes #<number>` only when the PR completes that issue; otherwise use
  `Refs #<number>` and leave remaining work on the issue. Keep active task status,
  decisions about scope, and follow-up work in GitHub issues, not a parallel
  Markdown task list. Historical plans remain reference material, not a backlog.
  Reviewers MUST verify the issue linkage and acceptance criteria before merge.
  **Evidence:** linked issue, PR body, and issue-specific acceptance evidence.
- **Commit coherent candidates.** Commits MUST contain only intended, reviewed
  changes and describe their purpose accurately. Verify the staged diff; do not
  include unrelated user work, secrets, or transient artifacts. Commit/push only
  within the user's requested or established delivery scope. Do not amend,
  rewrite shared history, or bypass checks without explicit authorization.
  **Evidence:** staged diff, applicable check results, and meaningful commit history.
- **Use PRs as acceptance boundaries.** PRs MUST state purpose, scope, validation,
  limitations, and relevant decisions/standard version. Required checks and
  unresolved review findings MUST be visible; no self-approved exceptions or
  merges. Opening a PR does not authorize merging it. Merge requires explicit
  authority and satisfaction of configured protections; disclose absent controls.
  **Evidence:** candidate-specific checks, review resolution, and merge authority.
- **Deploy identified releases, not working trees.** Deployment MUST target an
  identified revision/artifact and named environment with authorization covering
  that action. Before promotion, verify applicable gates, configuration/secrets,
  operational readiness, and rollback/recovery plan. Destructive migrations or
  irreversible effects need explicit approval. Verify health after deployment
  and report the release, outcome, and any recovery performed; a successful
  deployment command alone is not success. PR approval/merge is not production
  deployment authorization unless an approved release policy explicitly says so.
  **Evidence:** release identity, authorization, readiness checks, recovery plan,
  and post-deployment observations.

## Amendments and exceptions

Changed obligations MUST bump this version and synchronize the marker in
[AGENTS.md](AGENTS.md) in the same PR. Major: incompatible obligations; minor:
new/materially refined obligations; patch: clarifications. Meaning-preserving
editorial changes need no bump. Record rationale and migration impact in the PR;
consequential decisions also need ADRs. Git preserves the history.

Resolve version mismatches before implementation. The current standard is
authoritative, not a remembered summary. Exceptions require prior approval,
affected requirement, scope, risk, rationale, and expiry/resolution trigger.
Architectural exceptions need ADRs; temporary debt needs a tracked record.
Agent agreement or tool output cannot approve exceptions.

Evidence belongs in existing code, tests, configuration, ADRs, or review reports,
not mandatory new documents. Documentation-only changes need no runtime tests.
Judge quality through defects, rework, debt, and comparable measurements, not
code volume, coverage alone, or agent consensus. Define denominators and separate
requirement changes from defects and approval waits from active effort.

## Rust references

Consult relevant sections of the [API Guidelines](https://rust-lang.github.io/api-guidelines/),
[Book](https://doc.rust-lang.org/book/), and [Reference](https://doc.rust-lang.org/reference/)
when needed; do not load them wholesale or treat them as blanket checklists.
Compilation alone does not establish design or domain correctness.
