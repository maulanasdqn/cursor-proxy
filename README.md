# Cursor Proxy

A Rust proxy server that exposes Cursor's `agent` CLI models through standard OpenAI and Anthropic API formats. Any LLM client that speaks OpenAI or Anthropic protocol can use Cursor models as a drop-in backend.

Built with Axum 0.8, structured as a Cargo workspace following Clean Architecture principles.

## How It Works

The proxy sits between your LLM client and the Cursor `agent` CLI binary. When a request arrives:

1. The incoming message array is flattened into a single prompt string
2. The requested model name is mapped from Anthropic/OpenAI format to Cursor's internal ID (e.g. `claude-opus-4-6` becomes `opus-4.6`)
3. The `agent` CLI is spawned as a child process with the prompt
4. The response is formatted back into the expected API shape (OpenAI or Anthropic)

Both synchronous and streaming (SSE) modes are supported on all endpoints.

## Prerequisites

- Rust 1.82+ (edition 2024)
- The Cursor `agent` CLI binary installed and accessible in your PATH (or configured via env var)
- A valid Cursor subscription with access to the models you want to use

## Installation

```bash
git clone https://github.com/maulanasdqn/cursor-proxy.git
cd cursor-proxy
cargo build --release
```

The binary will be at `target/release/cursor-proxy-server`.

## Quick Start

```bash
# Start with defaults (binds to 127.0.0.1:8765)
cargo run -p cursor-proxy-server

# Or with configuration
CURSOR_BRIDGE_API_KEY=my-secret-key \
CURSOR_BRIDGE_PORT=9000 \
cargo run -p cursor-proxy-server --release
```

## API Endpoints

### `GET /health`

Returns server status and configuration.

```bash
curl http://localhost:8765/health
```

```json
{
  "ok": true,
  "version": "0.1.0",
  "workspace": "/tmp/cursor-proxy-xxxx",
  "mode": "ask",
  "defaultModel": "auto",
  "strictModel": true
}
```

### `GET /v1/models`

Lists available models from the Cursor CLI. Results are cached for 5 minutes.

```bash
curl http://localhost:8765/v1/models
```

### `POST /v1/chat/completions`

OpenAI-compatible chat completions endpoint.

```bash
# Synchronous
curl -X POST http://localhost:8765/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "claude-opus-4-6",
    "messages": [
      {"role": "user", "content": "Hello, who are you?"}
    ]
  }'

# Streaming
curl -X POST http://localhost:8765/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "claude-sonnet-4-6",
    "messages": [
      {"role": "user", "content": "Explain quicksort briefly"}
    ],
    "stream": true
  }'
```

### `POST /v1/messages`

Anthropic Messages API-compatible endpoint. Requires `max_tokens` in the request body.

```bash
curl -X POST http://localhost:8765/v1/messages \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "claude-opus-4-6",
    "max_tokens": 4096,
    "messages": [
      {"role": "user", "content": "What is the capital of France?"}
    ]
  }'
```

Streaming follows the Anthropic SSE protocol (`message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`).

## Configuration

All settings are controlled via environment variables.

| Variable | Default | Description |
|---|---|---|
| `CURSOR_BRIDGE_API_KEY` | _(none)_ | Bearer token for authenticating incoming requests. If unset, no auth is required. |
| `CURSOR_AGENT_BIN` | `agent` | Path to the Cursor `agent` CLI binary. Also checks `CURSOR_CLI_BIN` and `CURSOR_CLI_PATH` as fallbacks. |
| `CURSOR_BRIDGE_HOST` | `127.0.0.1` | Address to bind the server to. |
| `CURSOR_BRIDGE_PORT` | `8765` | Port to listen on. |
| `CURSOR_BRIDGE_DEFAULT_MODEL` | `auto` | Default model when none is specified in the request. |
| `CURSOR_BRIDGE_TIMEOUT_MS` | `300000` | Request timeout in milliseconds. |
| `CURSOR_BRIDGE_TLS_CERT` | _(none)_ | Path to TLS certificate file (PEM). Enables HTTPS when both cert and key are set. |
| `CURSOR_BRIDGE_TLS_KEY` | _(none)_ | Path to TLS private key file (PEM). |
| `CURSOR_BRIDGE_CHAT_ONLY_WORKSPACE` | `true` | Create a temporary directory per request so the CLI cannot read/write your real project files. |
| `CURSOR_BRIDGE_VERBOSE` | `false` | Enable debug-level logging. |
| `CURSOR_BRIDGE_STRICT_MODEL` | `true` | Remember the last explicitly requested model and reuse it when `auto` is requested. |
| `CURSOR_BRIDGE_SESSIONS_LOG` | _(none)_ | Path to a file for session logging. |
| `CURSOR_BRIDGE_WORKSPACE` | _(cwd)_ | Default workspace directory when `CURSOR_BRIDGE_CHAT_ONLY_WORKSPACE` is disabled. |
| `CURSOR_BRIDGE_APPROVE_MCPS` | `false` | Pass `--approve-mcps` flag to the agent CLI. |
| `CURSOR_BRIDGE_FORCE` | `false` | Pass `--force` flag to the agent CLI. |

## Model Mapping

The proxy translates standard Anthropic model names to Cursor CLI identifiers:

| Request Model | Cursor CLI ID |
|---|---|
| `claude-opus-4-6` | `opus-4.6` |
| `claude-sonnet-4-6` | `sonnet-4.6` |
| `claude-sonnet-4-5-20250514` | `sonnet-4.5` |
| `claude-opus-4-6-thinking` | `opus-4.6-thinking` |
| `claude-sonnet-4-6-thinking` | `sonnet-4.6-thinking` |
| `claude-sonnet-4-5-thinking` | `sonnet-4.5-thinking` |
| `claude-haiku-4-5` | `sonnet-4.5` (Cursor has no Haiku) |

Provider prefixes are stripped automatically (e.g. `anthropic/claude-opus-4-6` becomes `claude-opus-4-6` before mapping).

Any model name not in the mapping table is passed through to the CLI as-is.

## Project Structure

The project is organized as a Cargo workspace with 8 crates following Clean Architecture:

```
cursor-proxy-errors/           Centralized AppError enum
cursor-proxy-types/            Config, shared domain types
cursor-proxy-models/           Model name mapping and resolution logic
cursor-proxy-agent/            CLI process spawning, prompt building, workspace management
cursor-proxy-auth/             Bearer token authentication middleware
cursor-proxy-openai/           OpenAI-compatible endpoint
  application/                   ChatCompletions, ListModels use cases
  infrastructure/http/           Handlers, routes, DTOs
cursor-proxy-anthropic/        Anthropic-compatible endpoint
  application/                   Messages use case
  infrastructure/http/           Handlers, routes, DTOs
cursor-proxy-server/           Entry point, router composition, TLS setup
```

Dependencies flow strictly downward: `server` depends on feature crates, feature crates depend on `agent`/`models`/`errors`/`types`, and the leaf crates (`errors`, `types`) have no internal dependencies.

## Using with LLM Clients

### As an OpenAI-compatible backend

Point any OpenAI-compatible client at `http://localhost:8765/v1`:

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8765/v1",
    api_key="your-bridge-api-key",  # or any string if no auth configured
)

response = client.chat.completions.create(
    model="claude-opus-4-6",
    messages=[{"role": "user", "content": "Hello"}],
)
print(response.choices[0].message.content)
```

### As an Anthropic-compatible backend

```python
import anthropic

client = anthropic.Anthropic(
    base_url="http://localhost:8765",
    api_key="your-bridge-api-key",
)

message = client.messages.create(
    model="claude-opus-4-6",
    max_tokens=4096,
    messages=[{"role": "user", "content": "Hello"}],
)
print(message.content[0].text)
```

### With Claude Code

Claude Code uses the Anthropic API format natively. Set the `ANTHROPIC_BASE_URL` environment variable to point at the proxy, then run Claude Code as usual:

```bash
# Start the proxy (in one terminal)
CURSOR_BRIDGE_API_KEY=my-secret-key cargo run -p cursor-proxy-server --release

# Run Claude Code pointed at the proxy (in another terminal)
ANTHROPIC_BASE_URL=http://localhost:8765 \
ANTHROPIC_API_KEY=my-secret-key \
claude
```

Claude Code will send all requests to `/v1/messages` on your proxy, which translates them and routes them through Cursor's `agent` CLI.

If you want to lock a specific model rather than letting Claude Code negotiate:

```bash
ANTHROPIC_BASE_URL=http://localhost:8765 \
ANTHROPIC_API_KEY=my-secret-key \
claude --model claude-opus-4-6
```

You can also add these to your shell profile so Claude Code always uses the proxy:

```bash
# ~/.zshrc or ~/.bashrc
export ANTHROPIC_BASE_URL=http://localhost:8765
export ANTHROPIC_API_KEY=my-secret-key
```

If you are running the proxy with TLS enabled:

```bash
ANTHROPIC_BASE_URL=https://localhost:8765 \
ANTHROPIC_API_KEY=my-secret-key \
claude
```

### Custom headers

You can override the workspace directory per request using the `X-Cursor-Workspace` header:

```bash
curl -X POST http://localhost:8765/v1/chat/completions \
  -H 'X-Cursor-Workspace: /path/to/project' \
  -H 'Content-Type: application/json' \
  -d '{"model": "auto", "messages": [{"role": "user", "content": "Describe this project"}]}'
```

This is only effective when `CURSOR_BRIDGE_CHAT_ONLY_WORKSPACE` is set to `false`.

## Security Notes

- By default, each request gets its own temporary workspace directory that is cleaned up after the request completes. This prevents the CLI from accessing your real files.
- If you set an API key via `CURSOR_BRIDGE_API_KEY`, all requests must include a valid `Authorization: Bearer <key>` header.
- The server binds to `127.0.0.1` by default. Do not expose it to the public internet without TLS and authentication.
- TLS is supported natively via rustls. Set both `CURSOR_BRIDGE_TLS_CERT` and `CURSOR_BRIDGE_TLS_KEY` to enable HTTPS.

## License

MIT
