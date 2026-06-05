# Removal Spec A1-ai — AI / Agent / MCP surface

Scope: remove the AI/agent engine, Model Context Protocol, computer-use, voice
input, and the AI-routing input classifiers from Tarp, plus the
`warp_multi_agent_api` dependency. This is the largest single removal surface in
the fork (`app/src/ai` alone is ~222k LOC; ~629 `app/src` files reference AI).

This spec is file-level and sequenced so an implementer can execute mechanically.
It is consistent with `docs/05-removal-map.md` (Tier A/B/C) and the
tracked-vs-owned split in `docs/08-upstream-sync.md`. It deliberately does **not**
re-cover cloud/accounts/telemetry/code-editor except where they share an edge
with AI (those are sibling specs).

Editor disambiguation (project rule 1): the `editor` crate (command-line input
buffer) STAYS and is untouched here. Nothing in this spec edits `crates/editor`.

---

## 1. Crates to delete wholesale (Tarp-owned, Tier A)

All are workspace members via the `crates/*` glob in `Cargo.toml:2-5`, so deleting
the directory drops them from the workspace automatically — no `members` edit
needed. None are in `default-members` (`Cargo.toml:11-23`).

| Crate | Path | LOC | Notes |
|---|---|---|---|
| `ai` | `crates/ai/` | 25,853 | AI/agent engine. Consumes `warp_multi_agent_api`, `mcp`, `computer_use`, `rmcp`. Has a `[[bin]]` named `ai` (`crates/ai/Cargo.toml:2`). |
| `mcp` | `crates/mcp/` | 2,229 | MCP client/host. Depends on `cloud_object_models` (Tier-A cloud removal) and `rmcp`. |
| `computer_use` | `crates/computer_use/` | 4,313 | Agentic screen/file actions. Ships a `use_computer` bin (`crates/computer_use/Cargo.toml:9-11`). |
| `voice_input` | `crates/voice_input/` | 422 | Voice capture for AI dictation. |

Reverse-dependency check (who pulls these — see who must change in §3/§4):

- `ai` is depended on by: `crates/cloud_object_models/Cargo.toml`,
  `crates/onboarding/Cargo.toml`, `app/Cargo.toml`. Both crates are themselves
  Tier-A removals (cloud / accounts), so their `ai` edges die with them — no
  rewiring required, only ordering (remove this surface before/with them).
- `mcp`: `crates/ai/Cargo.toml`, `app/Cargo.toml`.
- `computer_use`: `crates/ai/Cargo.toml`, `app/Cargo.toml`.
- `voice_input`: `app/Cargo.toml` (behind the `gui`/`voice_input` features).

### Classifier crates — assessment: REMOVE (AI-adjacent, Tier C → A)

| Crate | Path | Role | Verdict |
|---|---|---|---|
| `input_classifier` | `crates/input_classifier/` | Heuristic + ONNX classifier that decides command-vs-natural-language so input can be routed to the AI agent. | Remove. Its only consumers in `app/` are AI input routing + a telemetry event. |
| `natural_language_detection` | `crates/natural_language_detection/` | Word-list/stemmer NL detector. Used **only** by `input_classifier` (`crates/input_classifier/Cargo.toml`) and by AI input-mode wiring in `app`. | Remove with `input_classifier`. |

Evidence these are AI-only, not terminal-core:
- `app/src/lib.rs:43` `mod input_classifier;` and `app/src/lib.rs:2042`
  `ctx.add_singleton_model(input_classifier::InputClassifierModel::new)`.
- `app/src/ai/blocklist/input_model.rs:11-12,92,835` — the classifier feeds
  `is_agent_follow_up_input` / agent input decisions.
- `app/src/terminal/view.rs:14554-14683` — `natural_language_detection` is gated
  entirely on `AISettings` (the `apply_natural_language_detection_setting` path);
  with `AISettings` gone the toggle has no meaning.
- `app/src/terminal/input/slash_command_model.rs:2` and
  `app/src/terminal/shared_session/shared_handlers.rs:4` consume `InputType` — both
  are themselves removal-adjacent (slash commands feed agent mode; shared_session
  is cloud).
- `app/src/server/telemetry/events.rs:2569` references `input_classifier::InputType`
  — handled by the telemetry spec, not here.

Risk: removing the classifier deletes the "type plain English, get a command"
affordance. That is an AI feature by definition; a plain terminal does not need
it. Confirm with product before deleting (the only judgment call in this spec).

---

## 2. Dependency removal — `warp_multi_agent_api` + `rmcp`

- `Cargo.toml:321` `warp_multi_agent_api = { git = ".../warp-proto-apis.git", rev = ... }`
  — remove once all consumers are gone. Consumers: `crates/ai` (deleted),
  `crates/persistence` (TRACKED — see §5), `crates/integration`
  (`crates/integration/src/test/agent_mode.rs:21`), `app`.
- `Cargo.toml:531-533` `[patch."https://github.com/warpdotdev/warp-proto-apis.git"]`
  stanza (already commented) — delete with the dep.
- `Cargo.toml:393` `rmcp = { version = "1.6" }` — remove once `crates/ai`,
  `crates/mcp`, and `app` no longer reference it (all three are AI surface).
- After dep removal: regenerate `Cargo.lock`, and update `about.toml` /
  `deny.toml` entries for `warp_multi_agent_api` and `rmcp` (license/audit specs).
- `crates/integration/src/test/agent_mode.rs` — delete the whole agent-mode
  integration test module and drop `warp_multi_agent_api` from
  `crates/integration/Cargo.toml`.

---

## 3. `app/` cargo feature flags to drop (in `app/Cargo.toml`)

Strategy (per doc 05): **flip the `default` set first** to get a shrinking,
buildable target, then delete dead code, then delete the feature definitions.

### 3a. Dependency declarations to remove
- `app/Cargo.toml:241` `voice_input = { workspace = true, optional = true }`
- `app/Cargo.toml:408` `ai = { workspace = true, features = ["test-util"] }`
- the `ai`/`mcp`/`computer_use`/`rmcp`/`input_classifier`/`natural_language_detection`
  lines in `[dependencies]` (grep `app/Cargo.toml` for each crate name).

### 3b. Feature definitions to delete (with line refs sampled)
AI/agent core: `agent_mode` (456), `agent_mode_computer_use` (457),
`agent_mode_debug` (458), `agent_mode_primary_xml` (459),
`agent_mode_pre_plan_xml` (460), `agent_onboarding` (461),
`agent_shared_sessions` (462), `ask_user_question` (463),
`command_predictor` (470), `agent_management_view` (685),
`agent_management_details_view` (686),
`interactive_conversation_management_view` (687),
`suggested_agent_mode_workflows` (690), `grep_tool` (699),
`agent_mode_evals` (718, list-valued), `cli_agent_rich_input` (750),
`list_skills` (751), `render_agent_mode_output_markdown` (803),
`ai_rules` (838), `external_agent_mode_context` (846),
`agent_management_popup` (863), `reload_stale_conversation_files` (865),
`ai_context_menu` (869), `at_menu_outside_of_ai_mode` (870),
`agent_decides_command_execution` (871), `conversation_filter` (875),
`ai_context_menu_commands` (876), `ai_context_menu_code` (878),
`conversation_artifacts` (891), `conversations_as_context` (892),
`conversation_api` (893), `sync_ambient_plans` (894),
`summarize_conversation_command` (917), `web_search_ui` (919),
`web_fetch_ui` (920), `agent_view` (927) + all `["agent_view"]`-deriving
features: `agent_view_block_context` (928), `agent_toolbar_editor` (932),
`agent_view_prompt_chip` (934), `agent_view_conversation_list_view` (943),
`inline_history_menu` (944), `inline_repo_menu` (945),
`inline_model_selector` (948), `inline_profile_selector` (949),
`inline_slash_commands` (731), `agent_tips` (931), `named_agents` (937),
`agent_harness` (975), `active_conversation_requires_interaction` (959),
`ai_resume_button` (452), `bundled_skills` (455).

MCP: `mcp_server` (772), `mcp_oauth` (773), `mcp_debugging_ids` (774),
`mcp_grouped_server_context` (918), `file_based_mcp` (957).

Computer-use / voice: `local_computer_use` (756),
`local_claude_codex_child_harnesses` (757),
`voice_input` feature (817 `voice_input = ["dep:voice_input"]`),
and remove `voice_input` from `gui = ["voice_input"]` (`app/Cargo.toml:705`) →
`gui = []`.

Ambient agents / cloud-agent (shared edge with cloud spec — coordinate):
`ambient_agents_command_line` (727), `ambient_agents_image_upload` (728),
`scheduled_ambient_agents` (729), `ambient_agents_rtc` (935),
`cloud_conversations` (933). Leave the ultimate ownership of `cloud_conversations`
to the cloud spec, but its code lives under `app/src/ai`, so it falls out here.

Analytics edge (telemetry spec owns these, listed for awareness):
`global_ai_analytics_banner` (696), `global_ai_analytics_collection` (697).

Also remove every `default` array entry (`app/Cargo.toml` `default = [...]`,
~line 1+) naming the above, e.g. `agent_mode`, `mcp_server`, `mcp_oauth`,
`agent_view`, `web_search_ui`, `web_fetch_ui`, `image_as_context`,
`grep_tool`, `file_retrieval_tools`, `ai_context_menu`, `ask_user_question`,
`named_agents`, `agent_harness`, `cli_agent_rich_input`, etc. (sampled at
default lines 2,7,16,19,24-25,29-30,33-34,36,39-49,68,78,80-81,83-84,89,
92-94,100-101,103,105-109,113-120,130,132,134,136,141,148,153,158,162).

> Note: many AI features (`file_retrieval_tools`, `image_as_context`,
> `skill_arguments`, `pr_comments_skill`, `markdown_mermaid`, `artifact_command`,
> `suggested_rules`, `trim_trailing_blank_lines`) appear in `default` but their
> definitions may be co-located with other clusters. Grep each by name; if the
> `[]` body and all `#[cfg(feature = "...")]` users are AI-only, delete; else
> defer to the owning spec. `markdown_mermaid` pulls the `mermaid-to-svg` git dep
> (doc 03) used mainly for AI markdown output — flag for the deps spec.

---

## 4. `app/src` code surface (Tarp-owned — divergence is expected here)

### 4a. Whole module trees to delete
- `app/src/ai/` — entire directory (~222k LOC). Includes `agent`, `agent_sdk`,
  `agent_management`, `agent_conversations_model*`, `agent_events`,
  `ambient_agents`, `mcp/` (file-based + templatable MCP managers + gallery),
  `computer-use` glue, `voice/`, `blocklist/` (the AI history/blocklist model),
  `llms.rs`, `skills/`, `facts/`, `outline/`, `predict/`, `document/`,
  `conversation_*`, `restored_conversations.rs`, `harness_*`, `local_harness_setup.rs`,
  `request_usage_model.rs`, `execution_profiles/`, `cloud_agent_*`,
  `codebase_auto_indexing.rs`, `connected_self_hosted_workers.rs`,
  `metadata_project_rules.rs`, `persisted_workspace.rs`, `onboarding.rs`,
  `get_relevant_files/`, `generate_block_title/`, `generate_code_review_content/`.
- `app/src/ai_assistant/` — `app/src/ai_assistant/{mod,panel,requests,transcript,
  execution_context,utils,test_util}.rs` (~3.6k LOC; the `AskAI` panel).
- `app/src/voice/` — `app/src/voice/{mod,transcriber}.rs`.
- `app/src/input_classifier.rs` — the `InputClassifierModel` wrapper.
- `app/src/settings/ai.rs` (1,989 LOC) and `app/src/settings/ai_tests.rs` — the
  `AISettings` group (`settings/ai.rs:1`). Heavily referenced (114 files name
  `AISettings`), so this is the central blast radius. See §4d for the wiring.
- `app/src/settings_view/`: `ai_page.rs` + `ai_page_tests.rs`,
  `mcp_servers/` (whole dir: `edit_page.rs`, `list_page.rs`,
  `installation_modal.rs`, `update_modal.rs`, `server_card.rs`,
  `destructive_mcp_confirmation_dialog.rs`, `mod.rs`, `style.rs`),
  `mcp_servers_page.rs` + `mcp_servers_page_tests.rs`,
  `execution_profile_view.rs`, `agent_assisted_environment_modal.rs` +
  `_tests.rs`.
- `app/src/pane_group/child_agent/` — `{mod,hydration,restoration}.rs` (agent
  child panes).
- `app/src/terminal/cli_agent_sessions/` — whole dir (`event`, `listener`,
  `plugin_manager`, `mod.rs`): the local CLI-agent (claude/codex harness) session
  model.
- `app/src/search/ai_context_menu/` and `app/src/search/ai_queries/`.

### 4b. Module declarations to remove (the cut points)
- `app/src/lib.rs:4` `mod ai;`
- `app/src/lib.rs:43` `mod input_classifier;`
- `app/src/lib.rs:91` `mod voice;`
- `app/src/lib.rs:112` `pub mod ai_assistant;`
- `app/src/lib.rs:139-248` — the large block of `use ai::...` / `use crate::ai::...`
  re-exports and imports (e.g. `:139-143` agent todos/models, `:177-248` the
  active_agent_views / mcp / facts / harness / llms / skills imports,
  `:291` `AISettings`, `:296` `CLIAgentSessionsModel`). Delete each import and its
  use sites.
- `app/src/lib.rs:319-334` `fn determine_agent_source` + `:1131` its call site +
  `:703` the `warp_cli::CliCommand::Agent(...)` dispatch arm +
  `:1233-1301` the `multi_agent_conversations` restore plumbing +
  `:1514` skill-directory scanning + `:2042` the classifier singleton.
- `app/src/settings/mod.rs:2` `pub mod ai;` and `:41` `pub use ai::*;`.

### 4c. Menus / actions / commands (Tarp-owned)
- `app/src/app_menus.rs`: remove the New-AI menu —
  `:72` `make_new_ai_menu(ctx)` call, `:516-...` the `make_new_ai_menu` fn,
  `:5` `use ai::workspace::WorkspaceMetadata`, `:21`
  `use crate::ai::persisted_workspace::PersistedWorkspace`, and the
  `AISettings::is_any_ai_enabled` gating at `:989-1025` (agent-vs-terminal default
  session-mode branch — collapse to the terminal branch), plus
  `:1093-1095` `open_new_agent_tab_or_window`.
- `app/src/util/bindings.rs` (the `CustomAction` enum): remove variants
  `NewAgentModePane` (:121), `AttachSelectionAsAgentModeContext` (:123),
  `NewAgentTab` (:131), `ToggleConversationListView` (:134) and their keybinding
  arms (`:398-399`, `:424`, `:465`); and the `WarpAi` entry at `:801`/`:817`.
- `app/src/command_palette.rs` — remove AI/agent palette entries (grep for the
  removed `CustomAction` variants).

### 4d. `AISettings` blast radius (the hard part of Tier B)
`AISettings` is referenced in 114 `app/src` files. Recommended approach:
1. Delete `settings/ai.rs` and the `pub mod ai; pub use ai::*;` in
   `settings/mod.rs:2,41`.
2. Compile; for each error, the call is one of:
   - `AISettings::handle(ctx).read(...).is_any_ai_enabled(...)` → the surrounding
     block is an AI gate; delete the block (keep the else/terminal branch).
   - `default_session_mode == Agent` checks (e.g. `app_menus.rs:989`,
     `terminal/view.rs`) → collapse to the terminal-only path.
   - `apply_natural_language_detection_setting` (`terminal/view.rs:14554-14683`)
     → delete (depends on removed classifier).
3. `app/src/terminal/view.rs` holds ~1815 `crate::ai`/`AISettings`/agent/
   conversation references — it is the single largest Tarp-owned file to surgically
   thin. Expect this file (and `pane_group/mod.rs`, see below) to be the bulk of
   the manual effort.

### 4e. `pane_group` (Tarp-owned) — agent panes
`app/src/pane_group/mod.rs:25,45-65,121,127,140-167,212-216` imports the agent/
conversation/AI-document view models and declares `mod child_agent` (:167) plus
the `AmbientAgentViewModel` plumbing. Remove the agent pane kinds, the
`child_agent` module, ambient-agent restoration
(`app/src/pane_group/ambient_pane_restoration.rs`), and the AI imports; keep the
plain terminal/split pane kinds.

### 4f. Server / API surface (coordinate with cloud + telemetry specs)
- `app/src/server/...` `conversation_api` / agent server endpoints fall out with
  the `conversation_api` feature; the AI-shaped parts are removed here, the
  transport/auth parts belong to the cloud spec.
- `app/src/server/telemetry/events.rs:2569` (`input_classifier::InputType`) —
  delete with the telemetry spec once the classifier crate is gone.

---

## 5. ⚠ Reach into tracked-from-upstream crates (merge-conflict risk)

Project rule 2: minimize edits to tracked terminal-core crates. The good news
(verified): `warp_terminal`, `warp_core` (logic), `command`, `editor`,
`warp_completer`, `vim`, `syntax_tree`, `warpui*`, `ui_components`,
`markdown_parser`, `fuzzy_match`, `settings` have **zero** hard imports of
`ai`/`mcp`/`computer_use`/`warp_multi_agent_api`/`voice_input`. The AI surface is
almost entirely in `app/` and the deletable crates. The exceptions:

1. **`crates/persistence` (TRACKED) — hard entanglement. The one real conflict.**
   - `crates/persistence/src/model.rs:8` `use warp_multi_agent_api::response_event::stream_finished;`
   - `crates/persistence/src/model.rs:9` `use warp_multi_agent_api::{self as api};`
   - `crates/persistence/src/model_tests.rs:3` `use warp_multi_agent_api as api;`
   - The DB schema (`crates/persistence/src/schema.rs`) and model
     (`model.rs:893-1032` etc.) define `agent_conversations`, `agent_tasks`,
     `ai_document_panes`, `ai_memory_panes`, `ambient_agent_panes`, `ai_queries`
     tables plus `AgentConversationRecord`, `Vec<api::Task>`, conversation
     metadata. Removing `warp_multi_agent_api` forces edits here.
   - **Recommended minimal-divergence handling:** do NOT delete the agent tables/
     structs from the schema (that maximizes divergence from upstream). Instead,
     drop only the `warp_multi_agent_api` dependency and the few `api::` type uses
     in `model.rs` (replace the `Vec<api::Task>` / `stream_finished` usages with
     local stubs or remove the restore methods), keeping table definitions inert.
     Leaving unused tables costs nothing at runtime and keeps `persistence`
     close to upstream. Flag this file as a permanent known-divergence in
     `UPSTREAM_SYNC`. This is the single highest merge-conflict risk in the AI
     removal.

2. **`crates/warp_core/src/ui/icons.rs:137-138,468-469`** — `Icon::AgentMode` /
   `Icon::AmbientAgentMode` enum variants + svg paths. **Leave as-is** (unused
   enum variants are harmless; editing tracked code to delete them is pure
   merge-conflict cost with no benefit). The svg assets can be pruned by the
   branding/assets spec.

3. **`crates/warp_core` runtime context flags** —
   `crates/warp_core/src/context_flag.rs:119` `set_conversation_only()` and
   `execution_mode.rs:88-90` `can_fetch_agent_runs_for_management`. These are
   tracked-crate methods that reference agent concepts. **Leave the methods**
   (dead but inert); just stop calling them from `app/`. Do not edit warp_core.

4. **`crates/warp_features/src/lib.rs`** (the runtime `FeatureFlag` enum) has
   dozens of `AgentMode*`, `Mcp*`, `Conversation*`, `AgentView*`,
   `AgentManagementView`, etc. variants (`:82-573+`). These are **runtime** flags,
   distinct from the cargo features in §3. **Leave the enum variants** — they are
   inert once nothing reads them, and editing this crate is conflict cost. Just
   ensure their `default()`/config never enables anything (the cloud/telemetry
   spec may already disable via `set_conversation_only`/`set_essentials_only`).

5. **`crates/warp_cli` (the `warp` CLI binary's arg parser).** Doc 02 lists it as
   terminal-core; doc 08 does **not** list it as tracked-from-upstream, so it is
   effectively Tarp-owned-leaning. It carries large agent/MCP/skill subcommands:
   `agent.rs` (716), `mcp.rs` (86), `skill.rs` (193), `task.rs` (303), plus
   `share.rs`, `federate.rs`, `artifact.rs`, `schedule.rs`, `api_key.rs`,
   `harness_support.rs`, `provider.rs`, `model.rs`. These are pure clap arg
   structs (no `ai`/`mcp` crate deps). Remove the agent/mcp/skill/task/artifact/
   schedule subcommands and their `pub mod` lines in
   `crates/warp_cli/src/lib.rs:16-38` and the `CliCommand::Agent(...)` dispatch in
   `app/src/lib.rs:703`. Treat as Tarp-owned; this is a secondary surface, do it
   after the app/crate removal lands.

---

## 6. Other crates whose `ai` edge dies automatically (no rewiring)

These are Tier-A removals owned by sibling specs; listed so ordering is correct:
- `crates/cloud_object_models` uses `ai::document::AIDocumentId` / `ai::LLMId`
  (`notebook.rs:4`, `ai_execution_profile.rs:3`, `server_cloud_object.rs:189`).
  Removed by the cloud spec; its `ai` dep vanishes with it.
- `crates/onboarding` uses `ai::LLMId` (`model.rs:1`, `agent_onboarding_view.rs:3`,
  `slides/agent_slide.rs:1`, `bin/main.rs:5`). Removed by the accounts/onboarding
  spec.
- `crates/integration` agent-mode test (`src/test/agent_mode.rs`) — delete here.

---

## 7. Safe removal / sequencing order

Do this inside the broader doc-05 order (telemetry first, then code-editor); the
AI block itself sequences as:

1. **MCP + computer_use + voice_input first** (smallest, leaf-most): flip their
   features off in `app/Cargo.toml` default set (§3), delete `app/src/ai/mcp/`,
   `app/src/settings_view/mcp_servers*`, `app/src/voice/`, computer-use glue;
   delete crates `mcp`, `computer_use`, `voice_input`; remove `rmcp` (after `ai`
   too). Build.
2. **Classifiers**: confirm product decision, then delete `app/src/input_classifier.rs`,
   the singleton at `lib.rs:2042`, the NL-detection wiring in `terminal/view.rs`,
   and crates `input_classifier` + `natural_language_detection`. Build.
3. **AI core (the big one)**: flip all remaining `agent_*`/`ai_*`/`conversation*`
   features off; delete `app/src/ai/`, `app/src/ai_assistant/`,
   `app/src/settings/ai.rs`, `app/src/settings_view/ai_page*`,
   `pane_group/child_agent/`, `terminal/cli_agent_sessions/`,
   `search/ai_context_menu`/`ai_queries`; cut module decls and imports in
   `lib.rs`, `settings/mod.rs`, `app_menus.rs`, `util/bindings.rs`,
   `command_palette.rs`; thin `terminal/view.rs` and `pane_group/mod.rs` against
   compile errors (the `AISettings` blast radius, §4d). Delete crate `ai`. Build.
4. **`warp_multi_agent_api` removal**: handle `crates/persistence` minimally (§5.1),
   delete `crates/integration/src/test/agent_mode.rs` + its dep, remove the dep +
   `[patch]` from `Cargo.toml:321,531-533`. Build + `cargo test`.
5. **warp_cli subcommands**: remove agent/mcp/skill/task/etc. subcommands (§5.5).
   Build.
6. **Delete feature definitions** (§3b) now that no `#[cfg(feature=...)]` reads
   them; regenerate `Cargo.lock`; update `about.toml`/`deny.toml`.
7. Rebuild + `./script/presubmit`; run app smoke test.

Rebuild after each numbered step — every step leaves a compiling tree because the
surfaces are `#[cfg(feature)]`-gated.

---

## 8. Risks

- **`terminal/view.rs` (1815 AI refs) and `pane_group/mod.rs`** are the highest-
  effort manual edits. Both are Tarp-owned, so divergence is acceptable, but they
  are large and central — budget accordingly and lean on the compiler.
- **`persistence` schema (§5.1)** is the only tracked-crate change that's
  unavoidable; minimize it to dropping the `warp_multi_agent_api` dep, not the
  table definitions, and record it in `UPSTREAM_SYNC`.
- **Classifier removal is a product call**, not purely mechanical — it deletes the
  natural-language-to-command affordance.
- **Shared-edge features** (`ambient_agents_*`, `cloud_conversations`,
  `global_ai_analytics_*`, `markdown_mermaid`, `conversation_api`,
  `agent_shared_sessions`) straddle the cloud/telemetry/sharing specs; coordinate
  ownership so a feature isn't half-removed from two specs.
- **`gui = ["voice_input"]`** must be edited to `gui = []` or the build breaks
  after `voice_input` is deleted.
- Do **not** edit `crates/editor` (input buffer) or the renderer crates — none of
  them need it, and editing them is gratuitous merge-conflict cost.
