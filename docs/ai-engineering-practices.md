# AI-native engineering practices

Status: proposed on 2026-10-05. This is a workflow design, not installed agent,
skill, hook, CI, or security configuration.

## Outcome

Optimize for correct, maintainable changes and low rework, not generated code
volume. Combine Rust safeguards with explicit requirements, focused context,
repeatable tools, independent review, and human decisions at milestone boundaries.
No agent or skill guarantees quality.

## Repository knowledge

Use a concise root AGENTS.md for shared working agreements and a thin
.github/copilot-instructions.md for Copilot-specific discovery. Both should point
to one canonical coding-standards.md and the approved product/evaluation documents,
not duplicate large rule sets. Verify instruction loading in each actual client:
discovery and precedence differ between tools.

Record exact setup/check commands only after they work. Keep approved decisions
distinct from proposals. Update instructions when behavior or tooling changes;
stale instructions create repeated defects.

## Change workflow

1. Establish the acceptance criteria, affected contracts, non-goals, risks, and
   applicable quality gates. Ask about consequential ambiguity before coding.
2. Read relevant existing code and find prior patterns before proposing helpers.
   Use targeted search and avoid loading unrelated files into model context.
3. Implement one coherent vertical slice. Prefer explicit, small modules over
   speculative abstractions or generated scaffolding nobody needs.
4. For a bug, reproduce it with a failing test. For new behavior, derive tests from
   acceptance criteria, including negative and boundary cases. Do not merely
   encode the implementation's current output as the expected result.
5. Run focused checks while iterating, then all applicable PR gates. Report actual
   commands and results; skipped, unavailable, and failed checks remain explicit.
6. Review the diff for unintended behavior, scope drift, security boundaries,
   performance, and test adequacy. Fix coupled defects without unrelated cleanup.
7. Update directly affected docs, commit a reviewable change, and pause for the
   user at the agreed milestone. Human approval is not substituted by agent consensus.

## Agents: selective roles, bounded authority

| Role | When useful | Boundary |
| --- | --- | --- |
| Implementation owner | Each coherent change | Owns integration and validation; no overlapping writers |
| pre-committer | Milestone/PR review against standards | Read-only findings; does not rewrite or certify checks it did not run |
| Security specialist | Trust-boundary changes or explicit security review | Focus on evidence-backed risks; no vendor-system probing |
| Performance specialist | Measured regressions or complex resource behavior | Requires benchmark evidence, not speculative optimization |
| Research specialist | Substantial independent API/framework uncertainty | Return verified sources and bounded conclusions |

Do small lookups and simple edits directly. Delegate only work that benefits from
separate context. Give each agent the objective, constraints, relevant files,
acceptance criteria, allowed tools, stop conditions, and required evidence.

Parallelize independent read-only work or isolated changes; never assign concurrent
writers to the same files. The owner reconciles findings and runs integrated
checks. Set task budgets and stop unproductive retries. A second agent can share
the first agent's blind spots; review is not proof.

## Skills: reusable procedures, not an uncontrolled catalog

Add a skill when a concrete workflow repeats and needs instructions or scripts
beyond ordinary repository guidance. Initial candidates:

- Rust change validation: existing formatting, linting, and test commands.
- Jev contract integration: typed primitives, validation, version pinning,
  mocked failures, and request safety.
- Evaluation integrity: split isolation, label provenance, frozen configuration,
  metric definitions, and reproducible reports.

Prefer small project-scoped skills with clear triggers and example inputs/outputs.
Review third-party instructions and executable scripts before installation;
record their source and version. Do not install plugins or skills automatically,
grant broad permissions, or transmit repository/private data to external services.
Test each skill on a bounded fixture and keep its procedures aligned with CI.
Measure whether it reduces errors or repeated work before expanding it.

## Security by construction

Treat claim narratives, retrieved documents, tool responses, and model output as
untrusted data, not instructions. Prompt wording is not a security boundary.
Keep application authority in deterministic code and expose no payment,
coverage-adjudication, shell-execution, or emergency-action tools to the model.

Validate contracts and authorization independently of confidence. Use least
privilege, controlled egress, secret redaction, and synthetic test inputs.
Review CI permissions and third-party actions; pin action references to reviewed
commit SHAs. Use available secret scanning, dependency policy, and appropriate
static analysis, but verify availability rather than claim protection from plans.

Test instruction-like narratives and malformed outputs inside our application.
Never probe the provider's infrastructure. Escalate unauthorized actions or
sensitive-data exposure; do not suppress them to pass an evaluation.

## Performance and efficiency

Keep shared HTTP clients, bounded request sizes, bounded concurrency, deadlines,
and bounded retries. Avoid blocking work on async runtime threads and accidental
fan-out. Do not add caches without an explicit correctness and invalidation policy.

Measure end-to-end latency and retry cost before optimizing. Separate model
latency from parsing/routing overhead and record benchmark environment/settings.
Use deterministic local benchmarks for routing/validation when needed; paid
inference needs a separate budget and cannot be hidden in ordinary tests.

For coding efficiency, reuse validated commands, batch independent reads, and
run the smallest relevant test first. Do not trade correctness for fewer calls.
Track repeated failed approaches and stale instructions rather than endlessly
regenerating code.

## Technical debt controls and evidence

Record accepted debt with rationale, impact, owner, resolution trigger, and a
tracking reference. Do not create anonymous TODOs or unused abstraction layers.
Fix debt introduced by a change before declaring it complete unless explicitly
accepted. Dependency additions need a concrete use and maintenance justification.

Track outcomes with explicit denominators:

| Measure | Definition or interpretation |
| --- | --- |
| Escaped defects | Defects discovered after acceptance, by severity and affected change |
| First-pass gate success | Changes passing applicable gates on first complete validation / evaluated changes |
| Review rework | Corrective effort or revision rounds; distinguish defects from changed requirements |
| Change lead time | Agreed scope to accepted change; separate user/approval wait from active effort |
| Debt aging | Open accepted debt by age and impact, not raw TODO count |
| Coding resource cost | Agent/tool usage and elapsed effort per accepted milestone when observable |
| Performance regression | Comparable local benchmarks and approved runtime scorecard |

Do not equate lines generated, raw coverage, agent agreement, or fast completion
with quality. A single small sample cannot establish causal productivity gains.
Use observations to improve instructions and tests; no numeric productivity gates
are established yet.

## Implementation sequence

Confirm architecture, then author coding-standards.md and concise agent
instructions. Implement deterministic checks and CI before the first application
slice. Define pre-committer after its review contract is agreed; add skills only
as their workflows become concrete. Codex hook support remains unverified.

## Sources consulted

Official platform documentation describes capabilities, not proof of productivity
or safety. These recommendations apply that guidance to this project's scope.

- [GitHub repository instructions](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions)
- [GitHub agent skills](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills)
- [Codex AGENTS.md guidance](https://developers.openai.com/codex/guides/agents-md)
- [OWASP prompt injection risks and mitigations](https://genai.owasp.org/llmrisk/llm01-prompt-injection/)
