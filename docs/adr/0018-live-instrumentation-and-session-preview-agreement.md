# ADR 0018: Live instrumentation and session-scoped preview agreement

Date: 2026-10-08
Status: Accepted
Tracking issue: [17](https://github.com/niksacdev/zipclaim/issues/17)

## Context and user direction

The user requested actual terminal instrumentation, not just recorded event
lists, reusable UI components, a neon/animated ASCII Geek Mode without marketing
blocks, and one Data Protection acknowledgement per browser session. Customer
copy uses Reliable, Autonomous and Human in the Loop with Submit a Claim.
These are preview interaction/design changes, not approved insurance actions,
verified quality or a change to M01-M14 targets.

## Decision and alternatives

Embed official xterm.js 6.0.0 plus FitAddon 0.11.0 (MIT), dynamically loaded only
when Geek Mode mounts the terminal panels. Reviewed its official
[integration](https://xtermjs.org/docs/guides/using-addons/) and
[security](https://xtermjs.org/docs/guides/security/) guidance.
[React LogViewer](https://github.com/melloware/react-logviewer) provides virtualized
searchable logs and SSE, but xterm fits the requested terminal appearance with
official resizing support and no shell connection. Static preformatted event
lists cannot meet the live requirement; OS stdout/shell forwarding would expose
unreviewed content and unnecessary authority.

A tracing layer forwards only project-known structured request, workflow and
assessment/decision lifecycle events and allowlisted metadata. It excludes
message bodies, keys, raw vendor payloads and arbitrary log messages. Structured
workflow events already identify the actual planner/decision actor and model,
so no invented LLM call or stage is required. Live diagnostics occur before
durable recording; they are not the workflow audit ledger.

The shared operator key protects `GET /v1/operator/telemetry` with the same
loopback browser-origin boundary. Use fetch-stream SSE so authorization stays in
a header, not an EventSource URL. A 256-event ring and broadcast buffer, 4-client
limit, 15-minute stream session, keepalives, explicit lag frames and no automatic
reconnect bound memory and connections. Recent replay starts at a disclosed
sequence; process restart discards this diagnostic history. This is not a
multi-tenant production logging service.

The browser validates/bounds stream frames, retains at most 256 live entries
and caps terminal scrollback. xterm receives plain, control-stripped text only:
no terminal stdin, WebSocket shell, link addon or OSC execution. Browser metadata
clears and its request aborts on disconnect, persona switch or Geek Mode exit.
Recorded workflow inspection remains available independently.

Geek Mode changes tokens to a readable neon terminal theme, uses a compact
animated ASCII header (respect reduced motion), hides marketing hero/journey
blocks and brings the claim test console to the top. Normal mode remains white.
The Experimental Preview notice is the single limitation/disclosure surface.
The global Geek Mode toggle uses original SVG spy-goggle illustrations: shaded
lenses when off and illuminated night-vision lenses when on, with a Geek Mode
caption. Accessible state and keyboard button behavior remain unchanged.
Geek Mode borders and restrained edge glows match the goggles' neon-green
lenses across cards, controls and terminals; normal-mode borders are unchanged.

The Data Protection acknowledgement is shared across Customer and Assessment lab
and kept in versioned browser-tab session storage, not local storage. It covers
OpenAI, Jev or OSS models as deemed appropriate by the solution; only actually
configured providers are used, and no OSS adapter is added by this copy change.
Agreement survives input/provider/persona changes and refresh; no keys or messages
are stored there. Blocked storage produces an explicit page-lifetime limitation.
The acknowledgement is not authentication or legal insurer consent; server
authority checks remain unchanged.

## Consequences, verification and follow-up

### 2026-10-08 logo and sample refinement

The user subsequently requested the original ZipClaim logo in Geek Mode instead
of ASCII; the ASCII header and its animation are removed. Both the primary claim
chat and Assessment lab share a JSON dataset of fictional, realistic questions
in Simple Claim, Multiple Queries in One and Complex Ambiguous Claims categories.
Clicking fills the editable message without submission; repeated clicks cycle
within that category. Samples are demonstration inputs, not held-out evaluation
data, coverage assertions or a promise that the bounded runtime supports each
real-world scenario. Sample changes invalidate the workflow request identity and
old result, preserve the session agreement, and are disabled during execution.
The examples ask for customer outcomes (repairs, reimbursement, policy updates,
payment relief and resolving disputed responsibility), not help wording a claim.
The public Galileo insurance row referenced in ADR0010 informs the multi-goal
servicing pattern; messages are original development examples, not imported
benchmark cases. Locked evaluation cases remain untouched.

Additional dependencies have a direct caller and small integration boundary;
quality and advisory checks apply to the changed candidate. Runtime tests cover
field redaction, control stripping, ring/connection bounds, access/origin checks,
live-before-completion delivery and explicit lag. Browser checks exercise actual
local baseline events through SSE and xterm, without vendor calls. Frontend tests
cover split/malformed/oversized streams, disconnect/late-frame isolation,
session acknowledgement and mode switches. Reduced-motion and responsive
observations accompany the candidate record on issue17.

No cloud deployment, OS terminal access, production identity, new provider calls,
claim processing or business-outcome claim is authorized. niksacdev owns review at
the first experimental walkthrough; production log retention/access, customer
identity, provider evaluation and label/effort/finance evidence remain separately
required. Engineering standard0.6.0 applies; prior exact-tree review does not
cover this changed transport surface.
