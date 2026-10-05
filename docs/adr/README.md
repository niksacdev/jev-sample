# Architecture decision records

ADRs preserve consequential project decisions for people and coding harnesses.
Read relevant accepted records before changing their affected behavior.

| ADR | Status | Decision |
| --- | --- | --- |
| [0001](0001-insurance-intake-scope.md) | Accepted | Synthetic insurance intake, not adjudication |
| [0002](0002-harness-skills-and-review-context.md) | Accepted | Harness implementation, same-context skills, separate-context pre-committer |
| [0003](0003-separate-principles-from-tools.md) | Accepted | Durable principles separate from replaceable tooling |

Use sequential IDs and descriptive names. Include date, status, context, decision,
alternatives, consequences, and verification/follow-up. Statuses are Proposed,
Accepted, Superseded, or Rejected. Update the index in the same change.

Keep accepted rationale intact. A replacement needs a new record with reciprocal
links and a Superseded status on the old record. Dated corrections and
implementation evidence may update an existing record without rewriting history.
Decisions are not proof of implementation; document unresolved follow-up.
