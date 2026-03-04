# Milestone 12 — AI + MCP Integration

**Crates:** `ember_ai`, `ember_mcp`
**Depends on:** Milestone 11 (Editor)
**Effort:** ~2 weeks
**Deliverable:** Pluggable LLM providers (cloud + local) for in-game AI features, and an MCP server that exposes the engine to external AI coding agents.

---

## Tasks

### Track A: `ember_ai` — In-Game LLM Integration

#### 12.1 LLM Provider Abstraction

- [ ] Define `LlmProvider` trait: `async fn complete(prompt, options) → Result<Response>`
- [ ] Define `LlmMessage` (role + content), `LlmOptions` (temperature, max_tokens, model)
- [ ] Define `LlmResponse` (content, token usage, finish reason)
- [ ] Implement provider registry — users register providers by name at startup

#### 12.2 Cloud Providers

- [ ] Implement `OpenAiProvider` — calls OpenAI-compatible APIs (also covers local servers like LM Studio, vLLM)
- [ ] Implement `AnthropicProvider` — calls Anthropic Messages API
- [ ] Support API key configuration via `AiConfig` resource or environment variables
- [ ] Implement retry logic with exponential backoff
- [ ] Implement streaming responses via `async Stream<Item = String>`

#### 12.3 Local Model Support

- [ ] Implement `OllamaProvider` — calls local Ollama REST API
- [ ] Auto-detect local Ollama installation on startup (optional)
- [ ] Support model listing and selection from available local models

#### 12.4 Async Runtime Infrastructure

The engine currently uses `std::thread::spawn` + channels for background work. LLM API calls and streaming responses require a proper async runtime.

- [ ] Add `tokio` (or `smol`) as a dependency — prefer `tokio` for ecosystem compatibility with HTTP clients
- [ ] Implement `AsyncRuntime` resource — wraps a shared `tokio::runtime::Runtime` handle
- [ ] Bridge async results back to ECS via channels (same pattern as `AssetServer`)
- [ ] Ensure the async runtime doesn't block the game loop

#### 12.5 AI Resource & Plugin

- [ ] Implement `AiServer` resource — manages active provider, request queue, response cache
- [ ] Implement `AiPlugin` — registers resources and background async systems
- [ ] Requests run on async background tasks, results delivered via channel (like AssetServer pattern)
- [ ] Implement `AiRequestHandle<T>` — typed async handle for pending AI responses

#### 12.6 In-Game AI Features

- [ ] Implement `AiAgent` component — attaches an AI "brain" to an entity
- [ ] Implement `AiPromptTemplate` — parameterized prompt with entity context injection
- [ ] Implement `DialogueSystem` — NPC conversation flow using LLM completions
- [ ] Implement `ContentGenerator` — procedural text/data generation for game content
- [ ] Rate limiting — configurable requests-per-second to prevent API abuse

---

### Track B: `ember_mcp` — MCP Server for External Tooling

#### 12.7 MCP Server Core

- [ ] Implement MCP server using JSON-RPC over stdio (standard MCP transport)
- [ ] Implement `initialize`, `tools/list`, `tools/call` MCP protocol handlers
- [ ] Implement `resources/list`, `resources/read` for engine state inspection
- [ ] Support concurrent tool calls
- [ ] Graceful error handling and MCP error responses

#### 12.8 Engine Tools (exposed to AI agents)

- [ ] `get_scene_info` — returns full scene hierarchy with entity names and components
- [ ] `spawn_entity` — create a new entity with specified components
- [ ] `delete_entity` — remove an entity by ID or name
- [ ] `set_component` — modify a component's fields on an existing entity
- [ ] `get_component` — read a component's current values
- [ ] `execute_system` — run a named system on demand
- [ ] `list_resources` — enumerate all ECS resources and their types
- [ ] `set_resource` — modify a resource value
- [ ] `get_viewport_screenshot` — capture the current frame as an image
- [ ] `run_command` — execute a predefined engine command (play, pause, step)

#### 12.9 Engine Resources (read-only inspection)

- [ ] `scene://entities` — live entity list with component summaries
- [ ] `scene://hierarchy` — parent-child tree view
- [ ] `config://render` — current render settings
- [ ] `config://input` — input bindings
- [ ] `perf://fps` — frame timing and performance metrics

#### 12.10 Editor AI Integration

- [ ] Natural language command bar in the editor (requires Phase 11 editor)
- [ ] AI-powered entity inspector — "explain this entity's behavior"
- [ ] AI-assisted scene builder — "create a forest with 20 trees"
- [ ] Code generation from natural language — generates system/component Rust code

#### 12.11 Demo & Testing

- [ ] Create `ai_demo` — showcases in-game AI dialogue with a local/mock provider
- [ ] Create `mcp_demo` — starts MCP server, demonstrates external agent controlling the scene
- [ ] Implement `MockProvider` for deterministic testing without API keys
- [ ] Unit tests for provider abstraction and request/response handling
- [ ] Integration test: MCP client → server → engine state modification → verification

---

## Exit Criteria

- [ ] `cargo test -p ember_ai -p ember_mcp` — all tests pass
- [ ] `ai_demo` runs with either a mock provider or Ollama, demonstrating in-game AI
- [ ] `mcp_demo` starts an MCP server that an external AI client can connect to
- [ ] At least 5 MCP tools work end-to-end (spawn, delete, get_scene_info, set_component, screenshot)
- [ ] Provider switching works at runtime (cloud ↔ local)
- [ ] No API keys required for core functionality (mock + Ollama paths work offline)
