# Engineering standards

This is the canonical team standard for architecture, design, implementation,
and verification. MUST and MUST NOT are mandatory. Supporting practices explain
these rules; they do not introduce competing standards or mandate tool brands.

## 1. Decisions and scope

- **ES-01:** Every change MUST have explicit acceptance criteria and non-goals
  before implementation. Consequential ambiguity or conflicting guidance MUST
  be resolved with the user, not hidden by assumptions.
- **ES-02:** Implementation and review MUST consult the ADR index and relevant
  accepted ADRs. Consequential architectural choices MUST be recorded before
  implementation. Replacements MUST supersede prior ADRs with reciprocal links.
  Decision status and implementation evidence MUST be updated in the same change.
- **ES-03:** This sample MUST remain synthetic insurance intake/triage. It MUST
  NOT decide coverage, liability, fraud accusations, payments, or emergency actions.
  A change to that boundary requires explicit approval and a new decision record.

## 2. Design and implementation

- **ES-04:** Domain behavior and deterministic routing MUST be independent of
  transport and provider representations. New abstractions and dependencies MUST
  have a current use, justified cost, and reviewed maintenance implications.
- **ES-05:** Domain concepts MUST use explicit types and exhaustive outcomes.
  External values MUST be validated beyond deserialization before domain use:
  completeness, allowed labels, finite numerical bounds, and consistency.
- **ES-06:** Errors MUST remain explicit. Technical failure MUST remain distinct
  from model uncertainty even when both route to review. Code MUST NOT fabricate
  successful defaults or hide failure through broad suppression.
- **ES-07:** Work and resources MUST be bounded: input size, concurrency, retries,
  and deadlines. Cancellation MUST be handled deliberately. Blocking operations
  MUST NOT run unchecked on async execution threads. Optimization claims MUST
  use comparable measurements.
- **ES-08:** Project code MUST use safe constructs and MUST NOT rely on unchecked
  production panics. Any exception MUST document its invariants and be explicitly
  reviewed. Exact arithmetic and date operations MUST remain deterministic.

## 3. Security and verification

- **ES-09:** External content and model outputs MUST be treated as untrusted data,
  not authority or executable instructions. Permissions MUST be enforced in code,
  independently of model confidence. Secrets/private data MUST NOT enter source,
  telemetry, ordinary tests, or unapproved external services.
- **ES-10:** Changes MUST have acceptance-derived behavioral evidence covering
  applicable boundaries, negative cases, and failures. Bug fixes MUST include a
  regression test where executable tests apply. Ordinary tests MUST be repeatable,
  use synthetic/mocked inputs, and MUST NOT incur paid inference.
- **ES-11:** Applicable reproducible formatting, static analysis, compilation,
  tests, and dependency checks MUST pass before acceptance. Evidence MUST identify
  the candidate revision and actual checks/results. Missing or failed checks MUST
  be disclosed; configuration alone MUST NOT be claimed as verified enforcement.
- **ES-12:** Evaluation MUST preserve held-out isolation, label provenance, frozen
  settings, and approved denominators. Results MUST NOT claim business gains,
  safety certification, or superiority beyond the evidence actually collected.

## 4. AI-assisted workflow and maintenance

- **ES-13:** The harness MUST own implementation and integration. Skills MUST be
  bounded procedures in its context, not substitutes for independent review.
  Delegation MUST have a bounded objective and MUST NOT create overlapping writers.
- **ES-14:** Pre-committer MUST use separate context, inspect the exact candidate,
  run applicable approved checks in an isolated surface, and review these rules
  and relevant ADRs. It MUST report findings and unperformed checks without source
  edits, commits, pushes, or auto-approval. Until configured, its absence MUST be
  disclosed; equivalent checks and human review MUST NOT be represented as an
  agent run. CI MUST independently repeat automated gates once implemented.
- **ES-15:** Changes MUST update directly affected documentation and decision
  evidence. Accepted debt MUST have rationale, impact, owner, tracking reference,
  and resolution trigger. Unused generated code and anonymous TODOs MUST NOT be added.
- **ES-16:** Delivery MUST use coherent learning milestones with relevant Rust
  explanations and runnable checks when code exists. Work MUST pause for user
  discussion at each agreed milestone.

## Compliance and exceptions

The harness and reviewer MUST map affected rules to evidence in the change
summary or review report. They MUST distinguish pass, fail, not applicable, and
not configured; a context file is guidance, not proof of compliance.

Exceptions MUST identify the rule, scope, rationale, risk, approval, and expiry
or resolution trigger. Approval MUST precede the deviation. Architectural
exceptions belong in an ADR; temporary debt needs a tracked record. Tool output
or agent agreement MUST NOT grant exceptions.

Tool/version/command choices MUST live in ADRs, verified runbooks, and executable
configuration, not in this constitution. See AGENTS.md for context-loading rules.
