# 05 — Removal Map (de-Warp)

This is the core of the fork effort. It maps where the removable functionality
(AI, cloud, accounts/auth, telemetry, code-editor) actually lives.

## The key insight

Removal is **feature-flag + `app/`-layer surgery, not crate deletion.** The
functionality is woven through `app/src` (2,093 files) and gated behind cargo
features, not isolated in the obviously-named crates.

### `app/src` reach of each surface (files referencing)

| Surface | `app/src` files | Signal |
|---|---|---|
| AI / agents | **629** | `warp_multi_agent_api` / `crate::ai` / `mod ai` |
| Auth / accounts / login | **356** | `login`, `sign_in`, `auth` |
| Telemetry / analytics | **315** | `sentry`, `telemetry`, `analytics` |
| Cloud objects | **253** | `cloud_object` |
| Session sharing | **79** | `session_sharing` |
| MCP | **65** | `mcp::` |
| Computer use | **54** | `computer_use` |
| Onboarding | **54** | `onboarding` |

These overlap, but the message is clear: AI/cloud/account code is *pervasive* in
`app/`, not quarantined.

## The 292 cargo feature flags

`app/Cargo.toml` defines **292** features. The `default` set has **~190** entries,
the overwhelming majority of which are removal targets. A representative slice of
default-on features:

- **AI/agents:** `agent_mode`, `agent_mode_computer_use`, `agent_management_view`,
  `agent_view`, `named_agents`, `agent_harness`, `ask_user_question`, `grep_tool`,
  `file_retrieval_tools`, `web_search_ui`, `web_fetch_ui`, `ai_rules`,
  `ai_context_menu`, `command_predictor`, `global_ai_analytics_collection`,
  `image_as_context`, `conversation_api`, `summarize_conversation_command`, …
- **Cloud/sharing/accounts:** `cloud_mode`, `cloud_conversations`,
  `cloud_environments`, `viewing_shared_sessions`, `shared_with_me`,
  `session_sharing_acls`, `agent_shared_sessions`, `api_key_authentication`,
  `team_api_keys`, `warp_managed_secrets`, `loginless_conversion`,
  `skip_firebase_anonymous_user`, `oz_identity_federation`, `handoff_*`,
  `ambient_agents_*`, `remote_codebase_indexing`, …
- **Code editor (Warp Code):** `vim_code_editor`, `tabbed_editor_view`,
  `code_find_replace`, `code_launch_modal`, `file_tree`, `create_project_flow`,
  `inline_code_review`, `code_review_*`, `revert_diff_hunk`, `get_started_tab`, …
- **MCP:** `mcp_server`, `mcp_oauth`, `file_based_mcp`, `mcp_grouped_server_context`.
- **Telemetry/crash:** `cocoa_sentry`, `crash_reporting` (pulls `sentry`,
  `sentry-log`, `minidumper`, `crash-handler`).

### Strategy implication
The lowest-risk path to a terminal-only Tarp is almost certainly **"reduce the
default feature set first, then delete the now-dead code"** rather than deleting
crates and chasing compile errors. Many surfaces are already `#[cfg(feature)]`-gated,
so flipping defaults gives a buildable, shrinking target at every step.

## Removal targets — three tiers

### Tier A — delete wholesale (crates that are purely removable)
`ai`, `computer_use`, `voice_input`, `mcp`, `cloud_objects`,
`cloud_object_client`, `cloud_object_models`*, `cloud_object_persistence`,
`firebase`, `warp_server_auth`, `warp_server_client`, `onboarding`,
`managed_secrets`, `managed_secrets_wasm`, `remote_server`, `serve-wasm`,
`isolation_platform`, `warp_web_event_bus` (assess).

*`cloud_object_models` is also where `warp-workflows` is pulled in — see Tier C.

Combined these are well over 50k LOC. But deleting them forces the `app/`-layer
work in Tier B, because `app/` references them everywhere.

### Tier B — `app/`-layer surgery (the real effort)
Unpick AI/cloud/account/telemetry from `app/src` (the 600+/300+/300+/250+ file
reaches above). Approach:
1. Flip the corresponding default features off.
2. Resolve compile errors module-by-module (delete dead UI surfaces, settings,
   command handlers, menu items).
3. Rebuild after each chunk; keep `warp_terminal`/`warp_core` compiling.

Specific high-risk integration points:
- `session-sharing-protocol`: 66 files in `app/` + 2 in `warp_terminal`.
- `warp_multi_agent_api`: 56 files in `app/` + `ai` + `persistence` (2).
- `app/src/settings/ai.rs` and the broader settings tree.
- Telemetry calls (315 files) — likely a cross-cutting `analytics`/`sentry`
  facade that can be stubbed to no-op first, then removed.

### Tier C — keep, but untangle
- **`command-corrections`, `warp-command-signatures`, `warp-workflows`** are
  terminal features to keep (see [`03-dependencies.md`](03-dependencies.md)).
- **`warp-workflows` is currently pulled via `cloud_object_models`** (a Tier-A
  deletion). Rewire workflows so they survive cloud removal (local-only workflows).
- `natural_language_detection` / `input_classifier`: assess — they feed AI input
  routing; the terminal may not need them once AI is gone.
- `node_runtime` / `warp_js`: assess — used by plugins/MCP; may become dead.

## ⚠ "editor" disambiguation (repeated because it matters)
- The **`editor` crate** ("Text editing for Warp") is the command-line **input
  buffer** — content, selection, multiline, render. **Keep it.**
- The removable **"code editor" / Warp Code** feature is the `*_code_editor` /
  `tabbed_editor_view` / `file_tree` / `code_review_*` feature cluster in `app/`,
  not the `editor` crate. Remove via feature flags, not by deleting `editor`.

## Suggested removal order (de-risked)
1. Telemetry (stub the analytics/sentry facade to no-op) — small, isolating.
2. Code-editor feature cluster (flag-gated, comparatively self-contained).
3. MCP + computer_use + voice_input.
4. AI core (`ai`, `warp_multi_agent_api`, agent_* features) — the big one.
5. Cloud + sharing + accounts (`cloud_*`, `session-sharing`, auth, onboarding, firebase).
6. Untangle and re-home `workflows`; drop now-dead `node_runtime`/`graphql`/etc.
7. Prune the 2 removed git deps from `Cargo.toml`; regenerate `Cargo.lock`.
