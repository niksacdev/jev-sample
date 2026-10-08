# ADR 0019: Guided, memory-only planner setup and recoverable UI errors

Date: 2026-10-08
Status: Accepted
Tracking issue: [17](https://github.com/niksacdev/zipclaim/issues/17)

## Context and user direction

The user rejected a disabled claim button accompanied only by "Open Geek Mode
for setup details." They requested an actionable setup panel and explicitly
selected credentials in API memory until restart. They also directed graceful
error recovery using Rust, TypeScript and architecture best practices.
This enables the preview, not measured business improvement or real claim actions.

## Decision

Missing or unreadable workflow availability shows an inline operator setup
panel independently of Geek Mode. The panel preserves the customer's editable
message and agreement. It collects the existing local operator key, OpenAI API
key and explicit model ID. Password fields are cleared after success or failure;
credentials are never put in browser storage, URLs, telemetry or SQLite.
Refresh availability recovers from a lost setup response or configuration in
another tab without repeating setup or inference automatically.

Rust owns authorization, allowed origin, JSON/body bounds, value validation and
the production provider endpoint. The new protected
`POST /v1/operator/workflows/setup` returns only public workflow availability,
with no-store caching. Typed setup errors distinguish invalid input, an already
configured planner and connection-client initialization failure. Authentication
and missing operator configuration remain explicit, with operator setup guidance.
This is administrator onboarding for a loopback experimental instance, not
production customer credential collection or a replacement identity system.

A shared `OnceLock<Arc<dyn Planner>>` admits one initialization. Startup settings
and the first valid runtime setup both freeze the planner for that API process;
concurrent or later writes cannot replace it during admitted/resumed work.
Existing configured decision providers and private workflow storage are unchanged.
Setup constructs the existing OpenAI Responses adapter with its fixed URL,
bounded transport and application-owned policy. It does not verify a key or model
against the vendor or make a paid inference call. The first explicitly submitted
workflow may still surface account/model errors through existing typed failures.

## Alternatives and consequences

Browser storage risks credential exposure. Writing .env or SQLite would conflict
with the user's memory-only choice. Editing environment and restarting alone
preserves security but fails the requested guided interaction. Replacing the
entire runtime or allowing arbitrary model switching could break in-flight
provenance and continuation; one-time initialization is sufficient.
Restart forgets runtime setup, while separately configured environment values
retain their existing startup semantics. Changing an installed planner requires
restart. OpenAI Decisions and Jev configuration still use existing startup
settings; this slice fills the missing planner prerequisite, not a universal
provider/secret-management console.

## Verification and engineering lesson

### 2026-10-08 user-approved setup refinement

The user requested **ZipClaim token** terminology, masked credentials for
OpenAI/Jev, consent in **Agree and save**, and an automatic post-save readiness
refresh before submission unlocks. After clarification, retain an explicit
OpenAI model-name textbox rather than guessing an Astra model identifier.
This supersedes the planner-only setup interaction above.

The request now includes `jev_api_key`. Empty strings identify already configured
connections; nonempty replacement credentials are rejected. Startup planner and
provider adapters remain immutable. A single `OnceLock<SetupConnections>` bundle
atomically publishes only the missing planner and/or Jev adapter after all
validation/client construction succeeds. This replaces the planner-only
OnceLock without introducing partial setup or provider replacement races.
Lookup always prefers startup adapters; continuations retain model provenance.
Both client constructors use existing fixed production URLs and policy deadlines.
Runtime Jev setup applies to the AI-native workflow, not the legacy Assessment lab.

The UI collects only missing credentials, all as password fields, and clears them
on success/failure. Agree and save incorporates the terms, saves missing
connections, then checks public options; only confirmed OpenAI/Jev readiness
completes acknowledgement and unlocks submission. A failed post-save refresh
offers an explicit availability check without resending credentials. Refresh
alone cannot accept terms if Agree and save has not been chosen. The session
acknowledgement still survives refresh; keys do not.
Alternatives: asking for unused existing keys risks misleading replacement
semantics; independent provider writes risk partial readiness; trusting only the
POST response fails the requested refreshed check. Configuration still makes no
vendor call and cannot guarantee real account/model access.

Rust regression checks cover access/origin rejection, missing authorization
configuration, malformed/oversized values, secret-free responses/database,
one-time setup, concurrent admission and fresh-process configuration loss.
TypeScript tests cover field clearing, explicit retry/refresh, busy states,
draft/agreement preservation and unlocking submission without automatic inference.
Offline browser checks exercise setup with dummy values only.

The observed defect was a missing recovery interaction, not permission to hide
failures, fabricate availability or retry business operations. Existing standard
0.6.0's failure, boundary, authority and verification rules remain unchanged.
Enforce the user's approved recovery expectation through scoped UI guidance and
executable regressions. Owner niksacdev reviews setup usability at the first
experimental walkthrough; provider account verification and production onboarding
remain separate authorized work. Independent review applies to this new candidate.
