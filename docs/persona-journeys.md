# Claim of Thrones: jobs and persona journeys

**Status:** proposed synthetic reference journeys; pending human review.
Read with [scope](product-scope.md), [requirements](product-spec.md) and
[metrics](metrics.md). These are accountable perspectives, not five UI tabs or
claims about implemented capability.

## Jobs, pains and alternatives

Jobs describe circumstance, motivation and desired outcome. Functional,
emotional and social dimensions are discovery prompts, not invented customer
psychology. [Public evidence and its limits](product-scope.md#evidence-and-discovery-assumptions)
ground communication, recourse and dependency questions; local workflow pains,
motivations and willingness to pay still need interviews and observation.

| Job / perspective | Circumstance -> motivation -> desired outcome | Functional / emotional / social hypotheses | Existing alternatives | Metrics |
| --- | --- | --- | --- | --- |
| J-C1 Claimant: report and contribute | Loss or evidence request -> establish facts without repetition -> acknowledged report and understood evidence needs | Accurate contribution / confidence information arrived / being heard | Phone/agent, portal, emailed documents, personal chronology | M03, M04, M08 |
| J-C2 Claimant: understand and respond | Waiting, coverage question or offer -> understand options -> informed response and correct fulfilment or explained no-payment | Status and basis / reduced uncertainty / respectful treatment | Status calls, written explanation, handler/representative | M01, M03, M04, M06 |
| J-C3 Claimant: challenge or return | Disagreement, missing payment or new evidence -> accountable reconsideration -> acknowledged review and explained result | Challenge and tracking / confidence disagreement is permitted / voice without coerced acceptance | Insurer review/complaints, representative, applicable dispute body | M04, M06, M07 |
| J-H1 Handler: establish the case | Assigned gap, contradiction or urgent cue -> investigate without reconstruction -> supported assessment or owned inquiry | Source assembly / confidence about uncertainty / accountability | Claims notes, policy documents, email, specialists, checklist | M01, M04, M05, M08 |
| J-H2 Handler: authorized judgement | Coverage/loss/proposal needs judgement -> apply current evidence and delegation -> recorded decision or escalation | Evaluate basis and authority / avoid unsafe reliance / defensible decision | Manual assessment, supervisor, specialist/legal referral | M01, M05, M06, M07 |
| J-O1 Operations: accountable flow | Unowned, ageing or transferred work -> prevent lost cases -> acknowledged ownership and next action | Manage work/waits / confidence nothing disappeared / service accountability | Queue review, spreadsheets, huddles, supplier chasing | M03, M05, M08 |
| J-O2 Operations: recover and improve | Failure, unknown fulfilment or quality regression -> recover without duplication -> reconciled state and quality-gated improvement | Recovery/cohort analysis / confidence in control / credible reporting | Runbooks, reconciliation, sampling, reassignment | M01, M06, M08, M09 |
| J-B1 Buyer/finance: insurer value | Funding/renewal decision -> separate attributable value from capacity -> defensible net-value decision | Full-cost comparison / avoid misleading investment / stewardship | Time study, expense records, controlled pilot, procurement | M01, M02, M10, M11 |
| J-B2 Buyer/commercial: viable contract | Pilot/pricing proposal -> align payer value and sustainable delivery -> agreed obligations or no-go | Price/cost fit / commitment confidence / credible promises | Existing platform, internal automation, outsourcing, vendors | M10, M12, M13, M14 |
| J-T1 Technical/risk: boundaries | New source/model/user/action -> prevent unauthorized consequence -> scoped, traceable operation | Access/authority verification / control under failure / auditability | Access controls, approval registers, controlled integrations | M01, M06, M08, M09 |
| J-T2 Technical/risk: safe release | Change or incident -> detect regression and contain risk -> evidence-backed release, rollback or stop | Scenario/version testing / avoid opaque degradation / assurance | Mocked tests, change review, incidents, independent audit | M01, M05, M06, M08 |

## Shared journey rules

Lifecycle: reported -> evidence gathering -> coverage review -> loss assessment
-> resolution proposed -> required decisions/responses -> fulfilment or explained
no-payment -> closed. Review may start before or after closure; reopening retains
prior decisions and returns to the affected stage.

Work states are separate: ready, working, waiting for claimant/reviewer/external
party, reconciling, failed or completed. Every wait/failure has a reason,
accountable owner, next action and resume condition. Waiting is not active handling.
Fulfilment states are not applicable, not authorized, pending, unknown, failed
or confirmed. Submission is not confirmation.

Evidence, assessment, proposal, decision and claimant response are distinct,
revision-bound records. Model suggestion is not fact or authority.
Customer explanations protect restricted material while preserving meaningful
basis and review options. Missing timing estimates are unavailable, not invented.
Named teams, response obligations and jurisdictional rights require approval.

## Reference scenarios

| Scenario / jobs | Journey and accountable owner | Required exit / requirements / metrics |
| --- | --- | --- |
| S01 Routine report to fulfilment; J-C1/C2, J-H1/H2 | Intake acknowledges; claims obtains evidence, coverage/loss basis, authorized decision and required claimant response; fulfilment owner confirms execution | Explained confirmed outcome and review channel; R01-R07, R10; M01-M04, M06 |
| S02 Incomplete/contradictory evidence; J-C1, J-H1 | Handler identifies purposeful gaps; claimant corrects or explains inability; handler retains versions and owns inquiry | Current evidence used; unavailable facts not fabricated; R02, R08, R09; M01, M04, M05 |
| S03 Urgent/out-of-scope complexity; J-H1, J-O1, J-T1 | Intake requests approved specialist attention; specialist acknowledges; operations retains ownership until then | Accurate limitations, no emergency-dispatch claim; R01, R08; M03, M05, M08 |
| S04 Disputed coverage; J-C2/C3, J-H2 | Authorized reviewer examines policy/evidence revisions and disagreement | Explained determination or owned inquiry, never confidence-based denial; R03, R05, R06, R10; M01, M04, M07 |
| S05 Disputed valuation/rejected offer; J-C2/C3, J-H2 | Handler investigates rejection/new evidence; authorized reviewer changes or maintains proposal with reasons | Fresh affected approvals/responses; silence/rejection not acceptance; R04-R06, R09; M01, M03, M04, M07 |
| S06 Explained no-payment; J-C2/C3, J-H2 | Authorized decision establishes basis; claimant receives explanation and review options | Explicit basis for closure; acknowledgement does not waive review; R03, R05, R06, R10; M01, M04, M07 |
| S07 Pending/failed/unknown fulfilment; J-C2/C3, J-O2, J-T1 | Execution owner tracks instruction; reconciliation owner investigates uncertainty and contains wrong/duplicate effects | No blind resend or closure on unknown delivery; R07, R08, R10; M06, M08 |
| S08 Review/reopening; J-C3, J-H2, J-O1 | Review owner acknowledges challenge/new evidence, applies independence rules and records reassessment | Prior decision retained; adjustment needs fresh authority/reconciliation; R07, R09, R10; M01, M06, M07 |
| S09 Duplicate/disconnect/concurrent edit; J-C1, J-H1, J-O2 | Exact repeat retrieves result; intake reviews uncertain duplicates; case owner reconciles conflict/reconnect | No unsafe merge, duplicate effect or silent overwrite; R01, R09; M04, M06, M08 |
| S10 Assessment/source/assignment failure; J-H1, J-O1/O2, J-T2 | Recovery owner retains context and chooses bounded retry/manual handling/acknowledged transfer | No invented source, erased failure or bypassed gate; R02, R08, R09, R11; M01, M05, M08 |
| S11 Inaccessible journey/missing monitoring; J-C1/C3, J-O1, J-T1 | Assisted-channel owner supports contribution/review; operations handles missing signals | Accessible recourse, explicit telemetry gaps, protected information; R06, R08, R11; M04, M07, M08 |
| S12 Pilot/commercial decision; J-B1/B2, J-O2, J-T2 | Claims/product/finance compare matched workloads and quality; procurement tests contract; vendor finance includes delivery/support | Continue/revise/stop with uncertainty; capacity not cash; R12-R15; M01, M02, M09-M14 |

## Architecture and contract coverage matrix

Product coverage checklist for [system design](system-design.md), not a storage,
API or agent-topology commitment.

| Coverage / scenarios | Jobs | Requirements | Metrics |
| --- | --- | --- | --- |
| Receipt/evidence integrity; S01-S03, S09-S10 | J-C1, J-H1, J-O1, J-T1 | R01, R02, R08, R09 | M03-M05, M08 |
| Coverage/valuation/authority; S01, S04-S06 | J-C2, J-H2, J-T1 | R03-R06, R09 | M01, M04, M05 |
| Fulfilment/recourse; S01, S06-S08 | J-C2/C3, J-H2, J-O2 | R07, R10 | M01, M06, M07 |
| Recovery/access/inspection; S03, S07, S09-S11 | J-O1/O2, J-T1/T2 | R08, R09, R11 | M03, M05, M08, M09 |
| Quality/insurer value/vendor viability; S12 | J-B1/B2, J-O2, J-T2 | R12-R15 | M01, M02, M09-M14 |

Human claims, customer/recourse, operations, finance and technical-risk reviewers
walk every scenario. Synthetic walkthroughs establish requirements coverage only;
customer effort, quality and financial value require separate measurement.
