# A8 — Cargo Feature Flags: Categorization & Minimal Terminal Default

**Scope.** This spec covers the **292 cargo features** declared in
`app/Cargo.toml` (`[features]` block, lines ~ `app/Cargo.toml:42`–`app/Cargo.toml:560+`).
It categorizes every feature as **KEEP / REMOVE / DEFAULT-CHANGE**, defines the
**minimal terminal-only `default` set**, and gives the verbatim replacement
`default = [...]` block. This is the single highest-leverage de-Warp artifact:
shrinking `default` is what makes the rest of the removal a buildable, shrinking
target instead of a delete-and-chase-compile-errors slog (consistent with
`docs/05-removal-map.md:53` and `docs/08-upstream-sync.md:59`).

This is **read-only analysis**. The only write is this file.

---

## 1. How feature gating works here (the mechanism that makes this safe)

There are two layers, and the split between them is what keeps tracked crates
clean:

1. **Cargo features** declared in `app/Cargo.toml` `[features]`. Most are empty
   markers (`foo = []`); a minority forward into subcrates or pull `dep:` crates.
2. **The `FeatureFlag` enum**, defined in the **tracked** crate
   `crates/warp_features/src/lib.rs` (enum + `RELEASE_FLAGS` at
   `crates/warp_features/src/lib.rs:956`). `warp_core::features` re-exports it
   (`app/src/features.rs:4` → `pub use warp_core::features::*`).
3. **The bridge** is `app/src/features.rs` (**Tarp-owned**, `app/src/**`). Its
   `enabled_features()` (`app/src/features.rs:16`) builds the runtime-enabled set
   by extending a list of `#[cfg(feature = "...")] FeatureFlag::Variant` entries
   (`app/src/features.rs:25`–~`350`). Each cargo feature toggles whether that
   enum variant is added to the live set.

**Key consequence — disabling a feature does NOT touch a tracked crate.** When
you drop a feature from `default`, the corresponding `#[cfg(feature=...)]` line in
`app/src/features.rs` simply compiles out, and ~225 `cfg!(feature=...)` gates in
`app/src` (measured: 225 gated sites) drop their code. The `FeatureFlag::Variant`
can stay defined in `warp_features` harmlessly — **we do not need to edit the
tracked enum to disable a feature.** This is why "shrink default first" is
low-conflict: the divergence lives entirely in `app/Cargo.toml` + `app/src/**`,
both Tarp-owned per `docs/08-upstream-sync.md:51`.

---

## 2. Strategy: shrink `default` first, then delete dead gated code

Sequenced, de-risked, each step independently buildable:

1. **Replace the `default` set** (`app/Cargo.toml:71`–`app/Cargo.toml:260`, 187
   entries) with the minimal terminal-only set in §6. This is one edit to one
   Tarp-owned file. After this, the AI/cloud/code-editor/MCP UI is compiled out
   but the source still exists — the build should still succeed because nearly
   everything is already `#[cfg]`-gated.
2. **Build + smoke-test.** Fix any non-gated call sites that referenced
   now-disabled symbols (expected to be small — most surfaces are clean-gated).
3. **Delete the now-dead gated code** module-by-module in `app/src/**` (the
   600+/300+/300+ file reaches from `docs/05-removal-map.md:14`), removing the
   `#[cfg(feature=...)]` blocks and their feature declarations.
4. **Delete the feature declarations** from `[features]` once no code references
   them, and prune the matching `FeatureFlag` enum variants from
   `app/src/features.rs` (Tarp-owned bridge) — leaving the tracked
   `warp_features` enum untouched where possible (see §5).
5. **Drop dead `dep:` deps** that only the removed features pulled (`sentry`,
   `voice_input`, etc.) and regenerate `Cargo.lock`.

The categorization below is the input to steps 1 and 4.

---

## 3. Categorization of all 292 features

Legend:
- **KEEP** — terminal-relevant; remains a declared feature (may or may not be in
  `default`).
- **REMOVE** — AI / cloud / accounts / telemetry / Warp-Code / orchestration;
  delete the declaration + gated code.
- **DEFAULT-CHANGE** — terminal-relevant but currently in `default` and should
  *stay declared*; the change is only its default on/off state (most KEEP
  features that are already off stay off).

In practice almost every KEEP feature is also a "default-change" in the sense
that we rebuild `default` from scratch; below, KEEP marks "survives as a
feature", REMOVE marks "delete entirely".

### 3a. KEEP — terminal-relevant features (and why)

These describe genuine terminal/renderer/input/UX behavior. Each gets a one-line
terminal justification.

**Rendering / images / display**
| Feature | Why keep (terminal-relevant) |
|---|---|
| `ligatures` | Font ligature rendering in the grid. |
| `kitty_images` | Kitty graphics protocol — inline images in terminal output. |
| `iterm_images` | iTerm2 inline-image protocol support. |
| `kitty_keyboard_protocol` | Kitty keyboard protocol — correct key reporting to TUIs. |
| `markdown_tables` | Markdown table rendering in block output. |
| `markdown_mermaid` | Mermaid diagram rendering in block output (terminal feature, not AI). |
| `blocklist_markdown_table_rendering` | Per-app opt-out of MD table render. |
| `blocklist_markdown_images` | Per-app opt-out of MD image render. |
| `editable_markdown_mermaid` | Mermaid block interaction. |
| `system_theme` | Follow OS light/dark theme. |
| `default_adeberry_theme` | Default theme selection. |
| `ui_zoom` | Zoom the UI (accessibility). |
| `resize_fix` | Alacritty cursor-reflow fix on window resize (`app/Cargo.toml`, comment). |
| `trim_trailing_blank_lines` | Output trimming behavior. |
| `render_continuous_block_selections_with_single_border` | Block selection rendering. |

**Input / editing / selection / keybindings**
| Feature | Why keep |
|---|---|
| `rect_selection` | Rectangular (block) text selection. |
| `richtext_multiselect` | Multi-cursor / multi-select in the input buffer. |
| `ime_marked_text` | IME composition (CJK input) — macOS marked text. |
| `selectable_prompt` | Allow selecting the prompt text. |
| `clear_autosuggestion_on_escape` | Esc clears the inline autosuggestion. |
| `allow_ignoring_input_suggestions` | Dismiss input suggestions. |
| `permanent_autosuggestion_hint` | Persistent autosuggestion hint UI. |
| `cycle_next_command_suggestion` | Cycle through command suggestions. |
| `partial_next_command_suggestions` | Partial-accept of next-command suggestion. |
| `validate_autosuggestions` | Validate suggestions before showing. |
| `grab_the_baton_editing` | Input handoff editing behavior. |
| `vim` mode features → `vim_code_editor` is REMOVE (see note) but core vim crate stays. | — |

**Completions (terminal completions, NOT AI)**
| Feature | Why keep |
|---|---|
| `classic_completions` | The shell-style completions engine. |
| `force_classic_completions` | Force classic engine. |
| `completions_v2` | Next-gen completions (forwards `warp_completer/v2`; see §5 — tracked-crate touch). |
| `command_palette_file_search` | File search in command palette. |
| `command_correction_key` | "did you mean" command correction keybinding (terminal feature; see `docs/05-removal-map.md:88` — command-corrections is KEEP). |
| `dynamic_workflow_enums` | Workflow argument enums (workflows are a KEEP terminal feature). |
| `workflow_aliases` | Aliases for workflows. |
| `am_workflows` / `team_workflows` | **Assess** — `am_workflows` = "agent mode workflows" → REMOVE; `team_workflows` is cloud → REMOVE. Listed here only to disambiguate from `workflow_aliases`. |

**Tabs / panes / windows / layout**
| Feature | Why keep |
|---|---|
| `tab_close_button_on_left` | Tab UI placement. |
| `new_tab_styling` | Tab visual styling. |
| `vertical_tabs` | Vertical tab layout. |
| `vertical_tabs_summary_mode` | Vertical tab summary. |
| `tab_configs` | Per-tab configuration. |
| `grouped_tabs` | Tab grouping. |
| `directory_tab_colors` | Color tabs by directory. |
| `drag_tabs_to_windows` | Drag tabs between windows. |
| `multi_workspace` | Multiple workspaces. |
| `undo_closed_panes` | Reopen closed panes. |
| `quake_mode` | Quake-style dropdown terminal. |
| `full_screen_zen_mode` | Distraction-free full screen. |
| `minimalist_ui` | Minimal chrome UI. |
| `global_search` | Search across blocks/output. |

**Shell / process / platform integration**
| Feature | Why keep |
|---|---|
| `shell_selector` | Pick the login shell. |
| `ssh_tmux_wrapper` | tmux integration over SSH. |
| `ssh_enable_host_denylist_in_settings` | SSH host denylist. |
| `ssh_drag_and_drop` | Drag-drop files over SSH. |
| `inline_ssh_banner` | SSH banner display. |
| `in_band_generators_ssh` | Completion generators over SSH. |
| `run_generators_with_cmd_exe` | Windows generator execution. |
| `msys2_shells` | MSYS2 shell support (Windows). |
| `local_tty` | Local PTY session (set in build.rs; core). |
| `local_fs` | Local filesystem APIs (forwards to several crates; see §5). |
| `remote_tty` | Remote PTY-over-server (assess — terminal-relevant SSH/remote shell, not cloud-account). |
| `prevent_sleep`-style behavior → handled by crate, no feature. | — |
| `toggle_bootstrap_block` | Shell bootstrap block visibility. |

**History / settings / persistence (local)**
| Feature | Why keep |
|---|---|
| `rich_history` | Rich command history. |
| `inline_history_menu` | **REMOVE** — gated on `agent_view` (`app/Cargo.toml`), it is the agent-view inline history; see REMOVE. Listed to disambiguate from `rich_history`. |
| `settings_file` | File-backed settings. |
| `alacritty_settings_import` | Import Alacritty config. |
| `sequential_storage` | Local sequential storage backend. |
| `multi_profile` / `profiles_design_revamp` | Terminal profiles. |
| `inline_profile_selector` | **REMOVE** — gated on `agent_view`; see REMOVE. |

**Build / profiling / infra (KEEP as declared, not in default)**
| Feature | Why keep |
|---|---|
| `autoupdate` | App self-update (terminal app concern). |
| `changelog` | Changelog display. |
| `oz_changelog_updates` | **REMOVE** — Oz (cloud orchestration) changelog. |
| `standalone` | Self-contained bundle build (forwards `warp_assets/standalone`). |
| `release_bundle` | Release-bundle build flag (forwards `warp_core/release_bundle`; tracked — see §5). |
| `runtime_feature_flags` | Runtime flag overrides (dev). |
| `integration_tests` / `test-util` | Test harness (forwards into tracked crates; keep for tests, never in default). |
| `dhat_heap_profiling`, `jemalloc`, `jemalloc_with_profiling`, `jemalloc_pprof`, `jemalloc_auto_heap_profiling`, `heap_usage_tracking`, `pprof_cpu_profiling`, `traces`, `recording_mode`, `record_app_active_events` | Profiling/diagnostics; keep declared, off by default. NOTE `heap_usage_tracking` and `cocoa_sentry` pull `crash_reporting` → see telemetry caveat in REMOVE. |
| `gui` | Groups GUI-only features (currently `gui = ["voice_input"]` — **rewire**: drop `voice_input`, see §5). |
| `extern_plist`, `embed_plist` (cargo-udeps ignore), `figma_detection` | `figma_detection` is REMOVE (design tool detection for AI context). `extern_plist` keep (macOS plist). |
| `plugin_host` | **Assess/KEEP-gated** — JS plugin host; pulls `rquickjs`+`warp_js`. Terminal completions-v2 depends on it transitively. Keep declared; off by default unless `completions_v2` is wanted. |
| `pluggable_notifications` | Desktop notifications (terminal-relevant). |
| `notebook_parameter` | Notebook/workflow parameter input. |
| `warpify_footer` | **REMOVE** — Warp branding/AI footer; see branding doc `docs/06`. |

### 3b. REMOVE — AI / agents

`agent_mode`, `agent_mode_computer_use`, `agent_mode_debug`,
`agent_mode_primary_xml`, `agent_mode_pre_plan_xml`, `agent_mode_evals`,
`agent_onboarding`, `agent_shared_sessions`, `agent_management_view`,
`agent_management_details_view`, `agent_management_popup`,
`interactive_conversation_management_view`, `agent_view`,
`agent_view_block_context`, `agent_view_conversation_list_view`,
`agent_view_prompt_chip`, `agent_toolbar_editor`, `named_agents`, `agent_tips`,
`agent_harness`, `agent_decides_command_execution`,
`external_agent_mode_context`, `suggested_agent_mode_workflows`, `am_workflows`,
`predict_am_queries`, `prompt_suggestions_via_maa`, `ask_user_question`,
`grep_tool`, `file_retrieval_tools`, `read_image_files`, `image_as_context`,
`selection_as_context`, `diff_set_as_context`, `drive_objects_as_context`,
`conversations_as_context`, `ai_rules`, `suggested_rules`, `ai_context_menu`,
`ai_context_menu_code`, `ai_context_menu_commands`, `at_menu_outside_of_ai_mode`,
`command_predictor`, `ai_resume_button`, `render_agent_mode_output_markdown`,
`fast_forward_autoexecute_button`, `web_search_ui`, `web_fetch_ui`,
`conversation_api`, `conversation_filter`, `conversation_artifacts`,
`summarize_conversation_command`, `summarization_cancellation_confirmation`,
`summarization_via_message_replacement`, `shared_block_title_generation`,
`reload_stale_conversation_files`, `incremental_auto_reload`,
`active_conversation_requires_interaction`, `fallback_model_load_output_messaging`,
`retry_truncated_code_responses`, `inline_slash_commands`, `inline_model_selector`,
`restore_prompt_on_inline_model_selector_search`, `inline_history_menu`,
`inline_repo_menu`, `inline_menu_headers`, `inline_profile_selector`,
`skill_arguments`, `list_skills`, `bundled_skills`, `oz_platform_skills`,
`cli_agent_rich_input`, `code_mode_chip`, `code_command` adjacents,
`context_window_usage_v2`, `gpt_configurable_context_window`,
`context_line_review_comments`, `v4a_file_diffs`, `transfer_control_tool`,
`cross_repo_context`, `full_source_code_embedding`, `codebase_index_persistence`,
`codebase_index_speedbump`, `search_codebase_ui`, `use_tantivy_search`,
`nld_classifier_v1/v2/v3`, `nld_heuristic_v1/v2`, `figma_detection`,
`pr_comments_slash_command`, `pr_comments_v2`, `pr_comments_skill`,
`github_pr_prompt_chip`, `rewind_slash_command`, `revert_to_checkpoints`,
`queue_slash_command`, `queued_prompts_v2`, `pending_user_query_indicator`,
`lsp_as_a_tool`, `local_computer_use`, `local_claude_codex_child_harnesses`,
`codex_notifications`, `codex_plugin`, `usage_based_pricing`,
`billing_and_usage_page_v2`, `solo_user_byok`, `custom_inference_endpoints`,
`global_ai_analytics_banner`, `global_ai_analytics_collection`,
`gpt_configurable_context_window`, `integration_command`, `artifact_command`,
`fork_from_command`, `agent_mode_evals`.

> **Update (2026-06-16, ADR-011):** `cli_agent_rich_input` was later
> **re-enabled** and added to the `default` set — it powers the **Ctrl-G**
> rich-input composer for detected CLI coding agents (Claude Code, codex, …), while
> the agent footer/chips stay off. It is therefore an **exception** to the 3b REMOVE
> categorization above. See [`../DECISIONS.md`](../DECISIONS.md) ADR-011.

> Note: `voice_input` (REMOVE; pulls `dep:voice_input`) and the whole
> `crates/ai`, `computer_use`, `mcp` trees are crate-level deletions covered by
> `docs/05-removal-map.md` Tier A; here we just stop enabling their features.

### 3c. REMOVE — MCP

`mcp_server`, `mcp_oauth`, `mcp_debugging_ids`, `file_based_mcp`,
`mcp_grouped_server_context`.

### 3d. REMOVE — Cloud / accounts / sharing / orchestration (Oz/HOA) / handoff

`cloud_mode`, `cloud_mode_from_local_session`, `cloud_mode_image_context`,
`cloud_mode_setup_v2`, `cloud_mode_input_v2`, `cloud_conversations`,
`cloud_environments`, `create_environment_slash_command`,
`cloud_object_initial_load`, `enforce_revisions_to_cloud_objects`,
`personal_cloud_objects`, `remote_codebase_indexing`, `remote_code_review`,
`viewing_shared_sessions`, `creating_shared_sessions`, `shared_with_me`,
`session_sharing`, `session_sharing_acls`, `agent_shared_sessions`,
`shared_session_long_running_commands`, `loginless_conversion`,
`skip_firebase_anonymous_user`, `skip_login`, `fast_dev` (pulls `skip_login`),
`api_key_authentication`, `api_key_management`, `team_api_keys`,
`team_features_override`, `team_workflows`, `warp_managed_secrets`,
`git_credential_refresh`, `git_operations_in_code_review`,
`oz_identity_federation`, `oz_handoff`, `oz_launch_modal`, `open_warp_launch_modal`,
`orchestration_launch_modal`, `orchestration_pill_bar`,
`orchestration_viewer_pill_bar`, `orchestration_viewer_streamer`,
`ambient_agents_command_line`, `ambient_agents_image_upload`,
`ambient_agents_rtc`, `scheduled_ambient_agents`, `sync_ambient_plans`,
`handoff_local_cloud`, `handoff_cloud_cloud`, `hoa_code_review`,
`hoa_notifications`, `hoa_onboarding_flow`, `hoa_remote_control`,
`open_code_notifications`, `transfer_control_tool`, `remote_server`-adjacent,
`simulate_github_unauthed`, `open_warp_new_settings_modes`, `warpify_footer`,
`avatar_in_tab_bar` (account avatar in tab bar — REMOVE), `welcome_tab`,
`get_started_tab`, `agent_onboarding`, `oz_changelog_updates`,
`open_warp_launch_modal`, `code_launch_modal`.

### 3e. REMOVE — Code editor / "Warp Code" cluster

(Per project rule: this is the removable cluster in `app/`, NOT the `editor`
crate.)

`vim_code_editor`, `tabbed_editor_view`, `code_find_replace`, `code_review_find`,
`code_review_save_changes`, `code_launch_modal`, `code_mode_chip`, `file_tree`,
`create_project_flow`, `projects`, `inline_code_review`, `auto_open_code_review_pane`,
`embedded_code_review_comments`, `context_line_review_comments`,
`file_and_diff_set_comments`, `revert_diff_hunk`,
`discard_per_file_and_all_changes`, `expand_edit_to_pane`,
`allow_opening_file_links_using_editor_env`, `linked_code_blocks`,
`get_started_tab`, `git_operations_in_code_review`, `remote_code_review`,
`hoa_code_review`, `configurable_toolbar`.

### 3f. REMOVE — Telemetry / crash reporting

`cocoa_sentry` (pulls `crash_reporting`), `crash_reporting` (pulls `dep:sentry`,
`dep:sentry-log`, `dep:minidumper`, `dep:crash-handler`, `warp_logging/crash_reporting`,
`ai/crash_reporting` — `app/Cargo.toml`), `log_expensive_frames_in_sentry`,
`send_telemetry_to_file`, `record_app_active_events` (assess — app-active event
telemetry; default off, recommend REMOVE).

> Telemetry caveat: `heap_usage_tracking` (`app/Cargo.toml`) pulls
> `crash_reporting`. Keep heap profiling only if you also rewire it off
> `crash_reporting`; otherwise it transitively re-adds `sentry`. Recommend
> leaving all of these out of `default` regardless (they already are).

### 3g. KEEP-declared-but-default-off (no change needed; never in our default)

Diagnostics/build features that are *already* not in upstream `default` and stay
that way: `dhat_heap_profiling`, `jemalloc*`, `pprof_cpu_profiling`, `traces`,
`recording_mode`, `runtime_feature_flags`, `standalone`, `release_bundle`,
`test-util`, `integration_tests`, `plugin_host`, `completions_v2`,
`force_classic_completions`, `system_theme`, `quake_mode`, `rich_history`,
`multi_workspace`, `selectable_prompt`, `ssh_*`, `msys2_shells`, etc.

---

## 4. The minimal terminal-only `default` set (rationale)

The current `default` has **187 entries** (`app/Cargo.toml:71`–`:260`); the vast
majority are AI/cloud/code-editor/orchestration. The new `default` keeps only the
terminal-essential, broadly-wanted UX features that are safe on for everyone.
Everything diagnostic, platform-specific (set by `build.rs`: `local_tty`,
`local_fs`), or opt-in stays *out* of default (still declared, opt-in via build
flags).

Features deliberately **kept** in default and why:
- Rendering correctness/UX users expect on: `ligatures`,
  `render_continuous_block_selections_with_single_border`,
  `kitty_keyboard_protocol`, `kitty_images`, `iterm_images`, `markdown_tables`,
  `markdown_mermaid`, `system_theme`, `ui_zoom`, `resize_fix`,
  `trim_trailing_blank_lines`.
- Input/selection: `rect_selection`, `richtext_multiselect`, `ime_marked_text`,
  `clear_autosuggestion_on_escape`, `allow_ignoring_input_suggestions`,
  `cycle_next_command_suggestion`, `partial_next_command_suggestions`,
  `validate_autosuggestions`.
- Completions/corrections (terminal, not AI): `classic_completions`,
  `command_correction_key`, `command_palette_file_search`.
- Workflows (local terminal feature): `workflow_aliases`, `dynamic_workflow_enums`.
- Tabs/panes/layout: `new_tab_styling`, `tab_close_button_on_left`,
  `vertical_tabs`, `tab_configs`, `directory_tab_colors`, `drag_tabs_to_windows`,
  `multi_workspace`, `undo_closed_panes`, `full_screen_zen_mode`,
  `minimalist_ui`, `global_search`.
- Shell/SSH integration: `shell_selector`, `ssh_tmux_wrapper`,
  `ssh_enable_host_denylist_in_settings`, `ssh_drag_and_drop`,
  `in_band_generators_ssh`, `run_generators_with_cmd_exe`.
- History/settings/profiles (local): `rich_history`, `settings_file`,
  `alacritty_settings_import`, `multi_profile`, `profiles_design_revamp`.
- App lifecycle: `autoupdate`, `changelog`, `pluggable_notifications`.

Excluded from default (KEEP-declared, opt-in): `completions_v2` (pulls JS host;
let it be opt-in until verified independent of AI), `quake_mode`, `system`
profiling, `plugin_host`, `remote_tty`, `msys2_shells` (Windows-only; gated by
target anyway).

> Validate each kept default against its `cfg!`/`FeatureFlag` site after the
> first build. A few "terminal" features may have hidden edges into AI types
> (e.g. anything referencing `conversation`/`agent` structs). If a kept feature
> fails to compile once AI is gone, demote it from default and fix or drop it.

---

## 5. Features that touch / forward into TRACKED-from-upstream crates (merge-conflict risk)

Per `docs/08-upstream-sync.md:40` tracked crates: `warpui*`, `warp_terminal`,
`warp_core`, `command`, `editor`, `warp_completer`, `vim`, `syntax_tree`,
`warp_features`, `warp_util`, `settings*`. Flag every feature whose definition
forwards a sub-feature or `dep:` into one of these — editing/removing it risks
diverging a tracked crate.

| Feature | Forwards into (tracked) | Risk / guidance |
|---|---|---|
| `completions_v2` (`app/Cargo.toml`) | `warp_completer/v2` (gates ~17 files in `crates/warp_completer/src/**`, all on `feature = "v2"`) | KEEP. Do **not** delete `warp_completer/v2`; just keep `completions_v2` opt-in. Removing it would mean editing the tracked completer. |
| `local_fs` (`app/Cargo.toml`) | `warp_core/local_fs`, `lsp/local_fs`, `persistence/local_fs`, `repo_metadata/local_fs`, `ai/local_fs` | KEEP `warp_core`/`persistence`/`repo_metadata` arms (terminal needs local FS). Drop only the `ai/local_fs` arm when `ai` is deleted — that is an edit to this `app/Cargo.toml` line, not to the tracked crate. `warp_core/src/paths.rs` gates on `local_fs` (tracked) — keep enabled. |
| `crash_reporting` / `cocoa_sentry` | `warp_logging/crash_reporting`, `ai/crash_reporting`, and `warp_core` has its own `crash_reporting` feature (`crates/warp_core/Cargo.toml`; gated in `crates/warp_core/src/errors.rs`, `errors/anyhow.rs`) | REMOVE telemetry. Removing the `ai/crash_reporting` arm is fine (ai is deleted). **`warp_core` and `warp_logging` keep their own `crash_reporting` feature defined** — just never enable it from app. Do not delete the tracked-crate feature (avoids diverging them); leave the variant unused. |
| `release_bundle` | `warp_core/release_bundle` (gated in `crates/warp_core/src/channel/state.rs`) + `rust-embed/debug-embed` | KEEP unchanged — pure build flag. |
| `integration_tests` / `test-util` | `warp_cli/*`, `warpui/*`, `warp_core/integration_tests`, `warp_server_client/*` | KEEP `warpui`/`warp_core`/`warp_cli` arms; drop the `warp_server_client` arms when that crate is deleted (edit to this `app/Cargo.toml` line only). |
| `agent_mode_evals` | `cloud_object_models/agent_mode_evals`, `warpui/log_named_telemetry_events`, `warp_logging/agent_mode_evals` | REMOVE entirely (AI evals). Forwards into `warpui` (tracked) and `warp_logging` — but only by *enabling* their features; deleting `agent_mode_evals` from app does not edit those crates. |
| `traces` | `warpui/traces` → `warpui_core/traces` | KEEP declared, off. No edit needed. |
| `standalone` | `warp_assets/standalone` | KEEP — build flag. |
| `plugin_host` | `warp_cli/plugin_host` (+ `dep:rquickjs`, `dep:warp_js`) | KEEP-gated. `warp_cli` is terminal-core-ish; keep the arm. |
| `gui` | `gui = ["voice_input"]` | **REWIRE**: `voice_input` is REMOVE (AI). Change to `gui = []` (or drop the feature). This edit is in `app/Cargo.toml` only — no tracked-crate edit. |
| `FeatureFlag` enum itself | `crates/warp_features/src/lib.rs` (tracked) + `RELEASE_FLAGS` at `:956` | **Important:** disabling features needs **no edit here.** The bridge `app/src/features.rs` (Tarp-owned) gates each variant with `#[cfg(feature=...)]`. Leave removed variants defined in `warp_features` to keep that crate upstream-shaped (minimizes conflict per `docs/08-upstream-sync.md:62`). Only prune the `#[cfg]` lines in `app/src/features.rs`. `RELEASE_FLAGS` lists `Autoupdate, Changelog, CrashReporting, ImeMarkedText, SshRemoteServer` — `CrashReporting` is telemetry; either edit `app/src/features.rs:22` to not extend `RELEASE_FLAGS`, or override the set in the Tarp bridge rather than editing `warp_features`. |

**Bottom line on tracked crates:** the only *forced* tracked-crate awareness is
(a) keep `warp_completer/v2`, `warp_core/local_fs`, `warp_core/release_bundle`,
`warp_core/crash_reporting`, `warp_logging/crash_reporting` *defined* but unused,
and (b) handle `RELEASE_FLAGS::CrashReporting` from the Tarp-owned bridge, not by
editing `warp_features`. No tracked crate needs a source edit to disable any
feature.

---

## 6. Proposed new `default` block (verbatim)

Replace `app/Cargo.toml:71`–`app/Cargo.toml:260` (the current 187-entry `default`)
with:

```toml
default = [
    # --- Rendering / display ---
    "ligatures",
    "render_continuous_block_selections_with_single_border",
    "kitty_images",
    "iterm_images",
    "kitty_keyboard_protocol",
    "markdown_tables",
    "markdown_mermaid",
    "system_theme",
    "ui_zoom",
    "resize_fix",
    "trim_trailing_blank_lines",

    # --- Input / selection / autosuggestions ---
    "rect_selection",
    "richtext_multiselect",
    "ime_marked_text",
    "clear_autosuggestion_on_escape",
    "allow_ignoring_input_suggestions",
    "cycle_next_command_suggestion",
    "partial_next_command_suggestions",
    "validate_autosuggestions",

    # --- Completions / corrections (terminal, not AI) ---
    "classic_completions",
    "command_correction_key",
    "command_palette_file_search",

    # --- Workflows (local) ---
    "workflow_aliases",
    "dynamic_workflow_enums",

    # --- Tabs / panes / windows / layout ---
    "new_tab_styling",
    "tab_close_button_on_left",
    "vertical_tabs",
    "tab_configs",
    "directory_tab_colors",
    "drag_tabs_to_windows",
    "multi_workspace",
    "undo_closed_panes",
    "full_screen_zen_mode",
    "minimalist_ui",
    "global_search",

    # --- Shell / SSH integration ---
    "shell_selector",
    "ssh_tmux_wrapper",
    "ssh_enable_host_denylist_in_settings",
    "ssh_drag_and_drop",
    "in_band_generators_ssh",
    "run_generators_with_cmd_exe",

    # --- History / settings / profiles (local) ---
    "rich_history",
    "settings_file",
    "alacritty_settings_import",
    "multi_profile",
    "profiles_design_revamp",

    # --- App lifecycle ---
    "autoupdate",
    "changelog",
    "pluggable_notifications",
]
```

This drops **~155 default entries** (from 187 → ~50), all of them in the AI /
cloud / accounts / orchestration / code-editor / telemetry surfaces. Every
remaining entry is a declared feature in `[features]`, so this block compiles as
soon as it replaces the old one.

> After applying: build. If a kept entry references a removed AI/cloud symbol via
> a non-`cfg`-gated path, demote it (delete from this block) and revisit during
> the per-module dead-code deletion (step 3 of §2).

---

## 7. Removal / sequencing order (feature-flag specific)

1. **Apply §6 `default` block** → build → fix non-gated call sites. (One
   Tarp-owned file edit; biggest single de-Warp win.)
2. **Rewire `gui = []`** (drop `voice_input`); trim the `ai/*` arms from
   `local_fs`, `crash_reporting`, `integration_tests`/`test-util` (only the arms
   pointing at to-be-deleted crates).
3. **Prune `#[cfg(feature=...)]` enum-extend lines** in `app/src/features.rs` for
   every REMOVE feature; handle `RELEASE_FLAGS::CrashReporting` here (don't edit
   `warp_features`).
4. **Delete REMOVE feature declarations** from `[features]` once no `app/src`
   code references them (do this in the same per-surface waves as the crate
   deletions: telemetry → code-editor → MCP/computer_use → AI → cloud/accounts,
   matching `docs/05-removal-map.md:103`).
5. **Drop now-unused `dep:` deps** (`sentry`, `sentry-log`, `minidumper`,
   `crash-handler`, `voice_input`, and `rquickjs`/`warp_js`/`command-signatures-v2`
   *iff* `completions_v2`/`plugin_host` are also dropped) and regenerate
   `Cargo.lock`.

---

## 8. Cross-cutting integration points & risks

- **Build.rs-set features** (`local_tty`, `local_fs`) are toggled by
  `app/build.rs`, not the `default` list — do not add them to default; keep them
  declared. `local_fs` forwards into tracked crates (§5) — keep its non-AI arms.
- **`gui = ["voice_input"]`** is the one feature that *enables* an AI dep from a
  GUI grouping. Must be rewired or the binary keeps `voice_input`.
- **Telemetry transitive re-entry:** `cocoa_sentry`, `heap_usage_tracking` both
  pull `crash_reporting` → `sentry`. Even if you keep heap profiling, scrub these
  edges or `sentry` returns. All are already non-default; keep them so.
- **`RELEASE_FLAGS`** (tracked `warp_features`) force-enables `CrashReporting` in
  release bundles via `app/src/features.rs:22`. Tarp release builds will re-enable
  telemetry unless the bridge stops extending `RELEASE_FLAGS` (or filters it).
  Handle in the Tarp-owned bridge, not the tracked crate.
- **`completions_v2` / `warp_completer/v2`**: a real terminal completions engine
  living in a tracked crate, gated on `feature = "v2"`. Keep it; do not let AI
  removal touch `crates/warp_completer`.
- **Hidden AI edges in "terminal" features:** features like `global_search`,
  `command_palette_file_search`, or workflow features may reference AI/cloud
  context types in non-gated code. The first post-§6 build surfaces these; demote
  any that won't compile.
- **`am_workflows` vs `workflow_aliases`/`dynamic_workflow_enums`:** "am" =
  agent-mode; REMOVE `am_workflows`, KEEP the plain workflow features. Easy to
  conflate by name.
- **`avatar_in_tab_bar`** looks like tab UI but renders the signed-in account
  avatar → REMOVE (account surface).
