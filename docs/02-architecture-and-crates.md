# 02 — Architecture & Crates

Workspace layout and a map of every crate, so we know what is core-terminal vs.
removable before touching anything.

## Workspace shape

- `Cargo.toml` declares a workspace of `crates/*` + `app/` (69 crates + the app binary).
- `resolver = "2"`, Rust toolchain pinned to **1.92.0** (`rust-toolchain.toml`).
- `[workspace.package]` defaults: `license = "AGPL-3.0-only"`, `publish = false`.
- `default-members` is a lean subset (the app + core crates) so `cargo build` doesn't
  compile the whole tree (excludes `serve-wasm`, `integration`, etc.):
  `app`, `channel_versions`, `command`, `editor`, `graphql`, `markdown_parser`,
  `sum_tree`, `warpui`, `warp_completer`, `warp_terminal`, `warp_util`.

## Scale

| Metric | Value |
|---|---|
| Rust LOC (`crates/` + `app/`) | 1,393,807 |
| Rust files | 3,411 |
| `app/src` files alone | 2,093 |
| Workspace crates | 69 (+ `app`) |
| Resolved packages (`Cargo.lock`) | 1,507 |

## Crate map

Verdict legend: **Core** = keep (terminal essential) · **Keep?** = terminal-useful, assess ·
**Remove** = AI/cloud/account/telemetry · **Infra** = build/test/support.

### Rendering & UI (Core — and the MIT-licensed bits)
| Crate | Role |
|---|---|
| `warpui`, `warpui_core` | Warp's custom GPU UI framework. **MIT-licensed** (the rest of the repo is AGPL). |
| `warpui_extras` | Extra UI widgets on top of warpui. |
| `ui_components` | Higher-level UI components. |
| `editor` | "Text editing for Warp" — the **command-line input buffer** (content/selection/multiline/render). ⚠ **Core to the terminal — NOT the removable code editor.** |

### Terminal core (Core)
| Crate | Role |
|---|---|
| `warp_terminal` | The terminal itself — the central crate. Heavily entangled with everything. |
| `warp_core` | Core terminal logic. |
| `command` | Command/block model. |
| `warp_completer` | Completions engine (consumes `warp-command-signatures`). |
| `warp_cli` | CLI argument parsing for the `warp` binary. |
| `warp_ripgrep` | Thin ripgrep wrapper for the CLI. |
| `warp_search_core` | Search. |
| `vim` | Vim keybindings/mode. |
| `syntax_tree`, `languages` | Syntax highlighting / language support. |
| `markdown_parser` | Markdown rendering (block output). |
| `fuzzy_match` | Fuzzy matching for menus. |
| `input_classifier` | Classifies input (command vs. NL, etc.). |
| `natural_language_detection` | NL detection (assess — feeds AI routing). |

### Data structures & utilities (Core/Infra)
| Crate | Role |
|---|---|
| `sum_tree`, `string-offset` | Rope/tree data structures for text. |
| `warp_util`, `simple_logger`, `warp_logging` | General utilities + logging. |
| `settings`, `settings_value`, `settings_value_derive` | Settings system. |
| `persistence` | Local persistence (also touches multi-agent API — see removal map). |
| `virtual_fs`, `warp_files`, `watcher` | Filesystem abstractions + file watching. |
| `asset_cache`, `asset_macro`, `warp_assets` | Asset bundling. |
| `field_mask`, `jsonrpc`, `ipc`, `websocket`, `http_client`, `http_server` | Plumbing/transport. |
| `handlebars` | Templating (vendored fork). |
| `prevent_sleep`, `app-installation-detection`, `repo_metadata` | OS/host utilities. |
| `node_runtime`, `warp_js` | JS runtime (for plugins/MCP — assess). |

### AI / agents (Remove)
| Crate | LOC | Role |
|---|---|---|
| `ai` | 25,853 | The AI/agent engine. Consumes `warp_multi_agent_api`. |
| `computer_use` | 4,313 | Agentic computer-use (screen/file actions). |
| `voice_input` | 422 | Voice input for AI. |
| `mcp` | 2,229 | Model Context Protocol client/host. |
| `input_classifier`, `natural_language_detection` | — | AI-adjacent; assess whether terminal needs them. |

### Cloud / accounts / sharing (Remove)
| Crate | LOC | Role |
|---|---|---|
| `cloud_objects` | 2,401 | Cloud-synced object model. |
| `cloud_object_models` | 3,461 | Schemas (consumes `session-sharing-protocol` + `warp-workflows`). |
| `cloud_object_persistence` | 1,006 | Local cache of cloud objects. |
| `cloud_object_client` | 372 | Cloud object client. |
| `firebase` | 145 | Firebase client utilities (anonymous user, etc.). |
| `warp_server_auth` | 1,491 | Server auth. |
| `warp_server_client` | 1,385 | Server/GraphQL client. |
| `warp_graphql_schema`, `graphql` | — | GraphQL plumbing for the backend. |
| `warp_web_event_bus` | — | Web event bus (assess). |
| `managed_secrets`, `managed_secrets_wasm` | — | Warp-managed secrets (cloud). |
| `remote_server`, `serve-wasm` | — | Remote/web-compiled server. |
| `onboarding` | 11,516 | First-run/account onboarding flows. |
| `isolation_platform` | — | Sandbox for cloud/CI agent runs. |

### Build / test infra (Infra)
| Crate | Role |
|---|---|
| `integration` | Integration test harness (not in default-members). |
| `channel_versions` | Release channel/version metadata. |
| `warp_features` | Feature-flag plumbing. |
| `lsp` | Language Server Protocol support. |
| `command-signatures-v2` | Next-gen completions (feature-gated). |

> **Note:** the verdicts above are first-pass classification from names, sizes, and
> dependency edges. Several "core" crates (`warp_terminal`, `persistence`,
> `cloud_object_models`) have AI/cloud code threaded *through* them — see
> [`05-removal-map.md`](05-removal-map.md). Removal is not a clean per-crate cut.
