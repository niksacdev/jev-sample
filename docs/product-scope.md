# ZipClaim: product scope

**Status:** proposed; pending human product, claims, customer/recourse, finance and technical-risk signoff.
This document defines product boundaries. [Persona journeys](persona-journeys.md)
define jobs and scenarios; [product requirements](product-spec.md) define
acceptance; [metrics](metrics.md) define measurement.
No real claims decision, payment, customer contact or deployment is authorized here.

## Customer problem and value proposition

Help insurers resolve claims with less avoidable handling effort and claimant
uncertainty, while preserving correct entitlement, accountable decisions and
effective review. The product covers the job from report to verified resolution,
not intake classification alone.

| Perspective | Pain to investigate | Desired outcome |
| --- | --- | --- |
| Claimant | Unclear status, repeated information requests, difficult explanations or challenges | Understand what is needed, receive an explained outcome and retain effective review |
| Claims handler | Reconstructing fragmented facts, unresolved evidence and unclear decision authority | Apply judgement with complete, traceable context |
| Claims operations | Stalled or unowned work, uncertain handoffs and execution failures | Keep work owned and recover safely across teams and dependencies |
| Buyer/finance | Savings cases that confuse capacity with cash or omit retained costs | Attributable, quality-preserving net value after all relevant costs |
| Technical/risk | Opaque decisions, excessive access and uncontrolled retries or changes | Inspectable operation with enforceable authority and recovery boundaries |

The initial buying hypothesis is an insurer claims-operations sponsor working
with finance, procurement and risk. Buyer identity, budget ownership and
willingness to pay remain unvalidated. Claimants are beneficiaries, not assumed
purchasers. Vendor revenue and insurer benefit require separate accounting.

## Evidence and discovery assumptions

| Public evidence | Product implication and limitation |
| --- | --- |
| [J.D. Power 2024 U.S. Auto Claims Satisfaction Study, syndicated release](https://insurance-canada.ca/2024/10/30/jd-power-us-auto-claims-satisfaction-study/) identifies communication, timing expectations and proactive updates as important; digital preferences differ by task | Test understandable status and access to people, not digital-only completion. Recently settled U.S. claims exclude theft-only, glass-only and roadside-only claims; not evidence about every claimant |
| [FOS annual complaints data 2024/25](https://www.financial-ombudsman.org.uk/businesses/resolving-complaint/our-insight/annual-complaints-data-insight-2024-25) identifies car/motorcycle insurance as its most complained-about insurance product | Include disagreement and effective recourse. Selected complaints are not a dissatisfaction rate among all policyholders |
| [FCA motor insurance claims analysis](https://www.fca.org.uk/publications/multi-firm-reviews/motor-insurance-claims-analysis) identifies vehicle complexity, supply-chain delays and labour pressures | Separate controllable handling effort from external repair delays and indemnity; software cannot remove every cost driver |
| [FCA claims-handling review release](https://www.fca.org.uk/news/press-releases/premium-hikes-driven-claims-costs-insurers-told-improve-claims-handling) identifies motor referral delays/costs; separate home/travel findings concern oversight and management information | Test handoff ownership and useful operational information; do not assert that home/travel findings establish motor prevalence |

These sources justify investigation, not universal dissatisfaction or product
effectiveness. Repeat-information burden, fragmented handler context, emotional
motivations and willingness to pay require representative interviews and task
observation. No local interviews, customer quotations, prevalence, savings or
credentialed expert approval are claimed.

## Proposed initial reference journey

A **synthetic motor vehicle-damage journey for insurer claims operations** is
the proposed first exercise: report, evidence, coverage review, loss assessment,
proposal, insurer decision, claimant response, fulfilment or explained
no-payment, closure and review. This bounded exercise tests the proposition;
it does not approve narrowing the whole vision to motor insurance.

Included:

- Invented English-language policy, evidence, valuation and fulfilment records;
  no real personal or financial data.
- Routine damage and exceptions: missing/contradictory evidence, urgent cues,
  disputed coverage/valuation, rejected/revised offers and unavailable dependencies.
- Repeated reports, uncertain duplicates, reconnect, conflicting edits,
  failed/unknown fulfilment, no-payment and reopening.
- Distinct claimant responses, insurer-authority decisions, operational controls
  and independently assessed outcome quality.
- Proposed category taxonomy: collision, theft, glass, weather, other and unclear;
  supported "other" is not missing information.
- Accessible status and review routes, with simulated assisted-channel support.
- Matched manual/assisted exercises and transparent insurer/vendor economic
  scenarios, not production ROI claims.

Bodily injury, complex liability, catastrophe handling, fraud investigation,
litigation and third-party recovery require specialist scope review. The exercise
may show an owned handoff, not specialist adjudication. Urgent cues request
appropriate human attention, not diagnosis, emergency dispatch or a monitored
emergency channel. Other insurance lines and general policy/customer/billing
servicing are not approved extensions of this core journey.

## Authority and jurisdiction

Jurisdiction, applicable law, policy wording, delegation, review rights and
response obligations remain approval decisions. Synthetic policies are examples,
not authoritative policy. Automation may prepare information and coordinate
bounded work; it has no autonomous coverage, liability, settlement, denial or
payment authority.

Claimant acceptance is separate from insurer authorization. Acknowledging
no-payment does not mean agreeing with denial or waiving rights. Model confidence,
employee title and dashboard access cannot create financial authority.
Before real use, approve jurisdiction-specific deadlines, consent,
accessibility, retention and reviewer-independence rules.

## Runnable sample versus target product

The React/Rust sample compares Code (keyword baseline) and Jev assessments of
`claim`, `policy_change`, `customer_details` and `billing` intents. Matching intents
produce review-required task plans; no-match requests clarification. No real
assignment, insurer change or completed claim is established. The LLM comparator
is disabled. Operator history is process-local, not durable claims storage.

Full-journey evidence, coverage, decisions and fulfilment are not implemented.
The [HTML mockup](design/intake-workbench.html) is scripted simulation; persona
tabs are not authentication. No live-provider validation or customer/business
improvement is established. See [README](../README.md) for executable boundaries.

## Commercial and approval gates

Subscription, usage and independently verified-outcome pricing are alternatives,
not a selected business model. Do not reward denial, underpayment or unverified
closure. Approve payer, price basis, delivery/support obligations, implementation
costs and reversal/dispute treatment before a pilot.

Test insurer net value after fees and retained labour, alongside sustainable
vendor delivery contribution under exceptions, retries and support demand.
Released capacity is not automatically cash; appropriate indemnity is not an
automation saving. Use [M01-M14](metrics.md) without invented targets; the
approved intake benchmark remains a separate [component evaluation](evaluation-design.md).

Before implementation commitments, named human reviewers approve supported
cases, jurisdiction, authority, quality rubric, recourse, measurement and economics.
Before a real pilot, additionally approve data access, authoritative sources,
integration ownership, readiness and containment procedures. A synthetic
demonstration, passing tests or faster inference does not satisfy these gates.
