import type { ConnectionCatalog } from "./contracts";

/** Mirrors the API's checked-in connection catalog for offline tests. */
export const connectionCatalog: ConnectionCatalog = {
  routers: [
    { id: "openrouter", label: "OpenRouter", needs_endpoint: false, endpoint_hint: null,
      help: "One OpenRouter key; usage is billed to your OpenRouter credits." },
    { id: "azure_foundry", label: "Azure Foundry", needs_endpoint: true, endpoint_hint: "https://your-resource.openai.azure.com",
      help: "Your Azure Foundry resource key and endpoint, with a deployment named gpt-6-astra." },
  ],
  planners: [{ id: "openai", label: "OpenAI", routers: ["openrouter", "azure_foundry"], note: null }],
  decisions: [
    { id: "jev", label: "Jev", provider: "jev", routers: ["openrouter"],
      note: "Available through OpenRouter; not in the Azure Foundry catalog." },
    { id: "openai_decisions", label: "OpenAI Decisions API", provider: "openai", routers: [], note: "Coming soon" },
    { id: "microsoft_decisions", label: "Microsoft Decisions", provider: null, routers: [], note: "Coming soon" },
  ],
};
