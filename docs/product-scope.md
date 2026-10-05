# Insurance Claim Agent scope

Status: approved on 2026-10-05.

## Business outcome

Help an intake team route incoming claims correctly and quickly, while keeping
ambiguous cases visible for human review. The hypothesis is less routing effort
without more misroutes, not autonomous insurance adjudication.

## Initial scope

- Synthetic English-language auto-insurance intake narratives.
- Incident categories: collision, theft, glass damage, weather damage, other/unclear.
- Assessment of urgent-assistance cues in the supplied narrative.
- Explicit review handling for missing, contradictory, or mixed-incident information.
- No images or real claimant information.

The labeling rubric must distinguish a valid "other" incident from an unclear
incident. Details of that rubric are part of the proposed evaluation design.

## Responsibility boundaries

| Component | Responsibility |
| --- | --- |
| Jev | Narrow semantic judgments with typed outputs and uncertainty |
| Rust | Input validation, exact calculations if needed, explicit routing/review policy |
| LLM comparator | Equivalent structured judgments, not an intentionally prose-only baseline |
| Human | Review uncertain cases and retain responsibility for consequential decisions |

Free-form summaries are outside the initial comparison. Any future summary
feature needs separate approval and evaluation.

## Exclusions

No coverage, liability, fraud accusation, payout, or eligibility decisions.
Urgency flags request expedited human review; they do not initiate emergency
response or financial actions.

Stock-price forecasting and autonomous credit decisions were not selected.
A brief reference to a Loan Analysis Agent was corrected by the user; no loan
workflow or loan metrics were adopted.

## Delivery approach

Use Rust for the backend and quality tooling. Teach through small changes with
explanations, runnable commands, expected results, and tests. Pause after each
logical milestone for discussion.

Rust's compiler helps with type and memory safety, but does not establish good
architecture, sufficient tests, correct domain logic, or reliable model outputs.
The project will use established Rust frameworks and practices, without a Python
comparison or a language-superiority claim. See
[engineering standards](../engineering-standards.md).
