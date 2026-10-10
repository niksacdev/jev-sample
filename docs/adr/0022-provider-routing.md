# ADR 0022: Provider routing through OpenRouter or Azure Foundry

Date: 2026-10-10
Status: Accepted
Tracking issue: [22](https://github.com/niksacdev/zipclaim/issues/22)

## Context and user direction

ADR 0020 made OpenRouter the single-key gateway. The user asked for two router
options, OpenRouter and Azure Foundry, with a provider model behind the scenes
that routes each request to the router the user selects. Setup is organised in
three sections: **Router** (OpenRouter or Azure Foundry), **Planner** (OpenAI
only) and **Decisions** (Jev, with OpenAI Decisions API and Microsoft Decisions
shown as coming soon). Adding a router or provider should need minimal UI change.

Jev (`typesafe/jev-1.13`) is not in the Azure Foundry model catalog; it is served
by TypeSafe directly and through OpenRouter. Azure returns token usage per call but
no per-call price; Cost Management is aggregate and delayed.

## Decision

- **Route table in code.** `src/workflow/connections.rs` owns which router can
  serve each choice. Today OpenRouter serves the OpenAI planner and Jev; Azure
  Foundry serves the OpenAI planner. The table is code, not data: a new route
  needs an adapter anyway, and keeping it beside the adapters keeps the catalog
  and the factory in parity (unit-tested).
- **Catalog-driven UI.** `WorkflowOptions.connections` (`ConnectionCatalog`) lists
  routers, planner choices and decision choices with their supported routers and
  notes. The web setup panel renders from it; `web/src/connectionPlan.ts` picks
  each choice's router: the primary router when it serves the choice, otherwise
  the first router that does. With Azure Foundry primary, Jev shows
  "via OpenRouter — not available on Azure Foundry" and needs an OpenRouter key.
- **Per-choice setup contract.** `ConnectionSetup { planner, decisions,
  credentials }` names each route and one credential per router used. Credentials
  must match the routers used exactly; unsupported routes, duplicate slots,
  unknown fields and invalid endpoints are rejected; already filled slots are
  never replaced. Keys stay in API memory only (ADR 0019).
- **Azure Foundry.** The user supplies the resource endpoint and key. Only
  `https://<label>.openai.azure.com` or `https://<label>.services.ai.azure.com`
  hosts are accepted (no userinfo, port, query or fragment); the path is ignored
  and requests go to the v1 Responses URL `/openai/v1/responses` with the
  `api-key` header. The deployment name from `config/azure-foundry.json` is sent as
  `model`, and the served model must match the configured model or a dated
  snapshot of it. `AZURE_FOUNDRY_ENDPOINT` and `AZURE_FOUNDRY_API_KEY` configure it
  at startup; `AZURE_FOUNDRY_PLANNER_DEPLOYMENT` and `AZURE_FOUNDRY_PLANNER_MODEL`
  override the defaults.
- **Estimated cost.** When a router does not report cost, cost is estimated from
  token counts and the checked-in list price and recorded as
  `cost_source: "estimated"`; OpenRouter-reported cost is `"reported"`. Geek Mode
  labels estimates. The `gpt-6-astra` price (input $10, output $50 per million
  tokens, Global Standard short context, effective 2026-09-01) comes from the
  Azure Retail Prices API. Cached-input discounts are ignored, so estimates err high.
- **Provenance.** Routed model identities are `openrouter:<model>` or
  `azure_foundry:<deployment>`; direct vendor keys keep bare model IDs. Resume
  compares identities, so runs recorded under the previous bare OpenRouter IDs no
  longer match and must be started again.
- **Startup precedence.** Direct `OPENAI_*` and `TYPESAFE_API_KEY` settings win;
  Azure Foundry then fills a missing planner; `OPENROUTER_API_KEY` fills whatever
  is still missing. Startup uses the same `RouterConnection` factory as setup.
- **LLM comparison baseline.** When an OpenRouter credential is supplied and the
  OpenAI slot is empty, it is filled with the LLM-as-decider baseline from ADR
  0020, preserving earlier behaviour. This is not the OpenAI Decisions API, which
  stays "coming soon" until an adapter exists.

## Consequences

- Users can keep the planner on Azure while Jev runs through OpenRouter; this
  needs two credentials.
- New routers or decision providers are added in the route table, the catalog
  and an adapter; the setup panel needs no structural change.
- Not verified with live calls: `gpt-6-astra` deployability in a given Azure
  region, strict `json_schema` behaviour on Azure, the served-model echo format,
  and OpenRouter's alpha Decisions API. Estimated cost is not an invoice.
- Related: [ADR 0019](0019-guided-memory-only-planner-setup.md),
  [ADR 0020](0020-openrouter-gateway.md).
