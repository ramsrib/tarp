# Removal Spec A4 — Code Editor ("Warp Code") feature cluster

Status: analysis only (no source changed). Evidence gathered on the repo at
commit `2bb3a04b`. This is the implementer-facing worklist for stripping the
**Warp Code** feature cluster: the in-app code editor, file tree, tabbed editor,
code find/replace, diff/PR code-review surfaces, project-creation entrypoints,
and the get-started tab.

Companion docs: [`../05-removal-map.md`](../05-removal-map.md) (tiers + ordering),
[`../08-upstream-sync.md`](../08-upstream-sync.md) (tracked-vs-owned path split),
[`../02-architecture-and-crates.md`](../02-architecture-and-crates.md).

---

## 0. Scope & the editor-crate disambiguation (read first)

This cluster is the **Warp Code** product surface — a VS-Code-style editor, file
tree, and a GitHub-Desktop-style diff/PR code-review pane living **inside the
terminal app**. It is unrelated to typing commands at the prompt.

| Concept | Where | Verdict | Why |
|---|---|---|---|
| **`editor` crate** (`crates/editor`) | tracked terminal-core crate | **KEEP** | The command-line **input buffer** (content/selection/multiline/render). Not part of Warp Code. |
| **`crate::code::editor`** (`app/src/code/editor/`) | Tarp-owned `app/` module | **REMOVE** | The in-app **code-file editor** (`CodeEditorView`, LSP-backed buffers, gutter, goto-line, find-in-file, vim handler for files). A different `editor` namespace entirely. |

Whenever this spec says "editor", it means `app/src/code/editor/` (the code-file
editor), **never** `crates/editor`. The `crates/editor` crate must not be touched.

### What is in scope (REMOVE)
- `app/src/code/**` — the code-file editor, file tree, diff viewer, inline diff,
  LSP integration, find-references, global buffer model (56 files).
- `app/src/code_review/**` — the diff/PR review pane, comments, git dialog
  (commit/push/create-PR), diff state (local + remote) (44 files).
- `app/src/coding_entrypoints/**` — create-project / clone-repo / project-button
  entry UI (5 files).
- `app/src/pane_group/pane/get_started_pane.rs` + `get_started_view.rs` — the
  Get Started tab.
- `app/src/coding_panel_enablement_state.rs` — gate state for the coding panel.
- `app/src/settings_view/code_page.rs` (+ `_tests`) — the "Code" settings page.
- `app/src/terminal/view/init_project/**` — the "initialize project / pick LSP"
  flow reachable from a terminal tab (3 files).
- The `lsp` workspace crate (`crates/lsp`) becomes dead once the cluster + AI are
  gone (see §7).

### What is OUT of scope here (other specs / KEEP)
- **AI / agents** — owns the heaviest *consumers* of this cluster (`app/src/ai/**`,
  see §5). Sequence this cluster *with or after* the AI removal; do not try to
  remove Warp Code while `ai/blocklist/**` still imports `CodeEditorView`.
- **Cloud / remote** — `app/src/remote_server/**` (remote code review, diff-state
  over the wire) is removed with the cloud spec; it depends on `code_review`.
- **`crates/editor`, `crates/command`, `crates/warp_completer`, `crates/vim`,
  `crates/syntax_tree`, `crates/languages`** — tracked terminal-core; KEEP.
- **"Open in external editor"** (`app/src/util/file/external_editor/**`,
  `app/src/settings_view/features/external_editor.rs`) — *partially* keep: the
  "open this file/link in VS Code/$EDITOR" action is a terminal convenience and
  stays; only the toggles that drive the **in-app** code panes
  (`OpenCodePanelsFileEditor`, `PreferTabbedEditorView`) get pruned. See §4.

---

## 1. Cargo feature flags

All flag definitions are in `app/Cargo.toml`. The cluster's flags are currently in
the `default` set (lines cited from the `default = [ ... ]` array, lines 503–676)
and each has an empty stub definition lower in the file.

### 1a. Flags to remove from the `default` array (`app/Cargo.toml`)
| Feature | `default`-array line | stub-def line |
|---|---|---|
| `code_find_replace` | 534 | 873 |
| `tabbed_editor_view` | 542 | 884 |
| `revert_diff_hunk` | 547 | 899 |
| `code_review_save_changes` | 548 | 900 |
| `create_project_flow` | 549 | 717 |
| `get_started_tab` | 550 | 714 |
| `file_tree` | 551 | 902 |
| `vim_code_editor` | 552 | 896 |
| `code_launch_modal` | 553 | 903 |
| `auto_open_code_review_pane` | 563 | 911 |
| `inline_code_review` | 566 | 912 |
| `code_review_find` | 574 | 910 |
| `remote_code_review` | 674 | 901 |
| `git_operations_in_code_review` | 675 | 989 |

### 1b. Adjacent flags to assess (likely also remove — code/diff-review-only)
Grep `app/Cargo.toml` for these; they are Warp-Code-adjacent and become dead once
the panes are gone:
- `embedded_code_review_comments` (684), `hoa_code_review` (638),
  `discard_per_file_and_all_changes` (562), `diff_set_as_context` (564),
  `v4a_file_diffs` (583), `context_line_review_comments`,
  `file_and_diff_set_comments`, `pr_comments_slash_command`, `pr_comments_v2`,
  `pr_comments_skill`, `github_pr_prompt_chip`, `search_codebase_ui`,
  `command_palette_file_search`, `linked_code_blocks`, `code_mode_chip`,
  `ai_context_menu_code`, `selection_as_context`, `expand_edit_to_pane`.
- `allow_opening_file_links_using_editor_env` (543) — **KEEP**: this is the
  external-editor convenience, not the in-app editor.

> Several of the §1b flags are equally claimed by the AI/cloud specs (PR-comment
> slash commands, diff-as-context). De-conflict ownership during sequencing; flip
> them off in whichever spec lands last.

### 1c. The `FeatureFlag` enum is in a TRACKED crate — do NOT edit it
The `FeatureFlag` *enum variants* (`VimCodeEditor`, `TabbedEditorView`,
`FileTree`, `CreateProjectFlow`, `GetStartedTab`, `CodeLaunchModal`,
`CodeFindReplace`, `RevertDiffHunk`, `CodeReviewSaveChanges`, `CodeReviewFind`,
`AutoOpenCodeReviewPane`, `InlineCodeReview`, `EmbeddedCodeReviewComments`,
`HoaCodeReview`, `RemoteCodeReview`, `GitOperationsInCodeReview`) live in
**`crates/warp_features/src/lib.rs`** (lines 320–873), a tracked terminal-core
crate. **Leave these variants in place.** Deleting them would diverge a tracked
crate for zero benefit and create future merge conflicts.

The compile-time→runtime bridge is `app/src/features.rs::enabled_features()`
(the `#[cfg(feature = "...")] FeatureFlag::X,` entries at lines 207–495). Because
a variant is only registered as enabled when its cargo feature is set, **removing
the feature from `default` (§1a) makes `FeatureFlag::X::is_enabled()` return false
at runtime** — no enum edit required. After deleting the `app/src` modules, also
delete the matching `#[cfg(feature)] FeatureFlag::X,` lines from
`app/src/features.rs` (that file is Tarp-owned, safe to edit).

---

## 2. File-level worklist — delete wholesale (Tarp-owned `app/src`)

These directories/files are the cluster proper. No tracked terminal-core crate
imports them (verified: `crates/warp_terminal|warp_core|editor|command|warpui|
warpui_core` contain zero references), so deletion is contained to `app/`.

### 2a. `app/src/code/**` — in-app code editor (56 files) — DELETE dir
Highlights (for the implementer's mental model):
- `app/src/code/editor/` — `CodeEditorView`, `model.rs`, `view.rs`,
  `view/vim_handler.rs` (file-editing vim, NOT `crates/vim`), `find/`,
  `goto_line/`, `element/gutter_button.rs`, `comment_editor.rs`,
  `embedded_comment.rs`, `diff.rs`.
- `app/src/code/file_tree/` — file tree model + view + inline-rename editing.
- `app/src/code/diff_viewer.rs`, `inline_diff.rs` — diff rendering widgets.
- `app/src/code/local_code_editor.rs` (+ `_wasm.rs`), `global_buffer_model.rs`,
  `editor_management.rs` (defines `CodeSource` — a widely-imported type, see §5),
  `buffer_location.rs` (defines `LocalOrRemotePath` — also widely imported, §5),
  `active_file.rs`, `opened_files.rs`, `view.rs` (the `CodeView` pane body).
- LSP glue: `language_server_extension.rs`, `language_server_shutdown_manager.rs`,
  `lsp_logs.rs`, `lsp_telemetry.rs`, `find_references_view.rs`.

### 2b. `app/src/code_review/**` — diff/PR review pane (44 files) — DELETE dir
- `code_review_view.rs` (+ tests + integration), `comment_list_view.rs`,
  `comment_rendering.rs` (defines `CommentViewCard` used by AI blocklist, §5),
  `code_review_header/`.
- `comments/` — `comment.rs`, `batch.rs`, `convert.rs`, `flatten.rs`,
  `diff_hunk_parser.rs`, `pending_imported.rs` (defines `ReviewComment`,
  `AttachedReviewComment*`, `ReviewCommentBatch`, `CommentId` — imported by AI, §5).
- `diff_state/` — `local.rs`, `remote.rs`, `mod.rs`, `error.rs` (defines
  `DiffMode`, `DiffStats`, `LocalDiffStateModel`, `GitDeltaPreference`).
- `git_dialog/` — `commit.rs`, `pr.rs`, `push.rs` (the commit/push/create-PR UI).
- `git_status_update.rs` (`GitRepoStatusModel`, `GitStatusUpdateModel` — imported
  by AI context model + terminal `input_tests`), `telemetry_event.rs`
  (`CodeReviewTelemetryEvent`, `CodeReviewPaneEntrypoint` — imported in several
  terminal/AI files), `context.rs`, `diff_menu.rs`, `diff_selector.rs`,
  `find_model.rs`, `hidden_lines.rs`, `editor_state.rs`,
  `file_invalidation_queue.rs`, `scroll_preservation.rs`, `diff_size_limits.rs`.
- `GITHUB-DESKTOP-LICENSE` — vendored license text for the diff view; delete with
  the dir (and remove any attribution wiring — see [`../04-licensing.md`]).

### 2c. `app/src/coding_entrypoints/**` (5 files) — DELETE dir
`mod.rs`, `create_project_view.rs`, `clone_repo_view.rs`, `glowing_editor.rs`,
`project_buttons.rs`. `project_buttons::init(ctx)` is called from `app/src/lib.rs`
— remove that call (§3).

### 2d. Other standalone files — DELETE
- `app/src/pane_group/pane/get_started_pane.rs`
- `app/src/pane_group/pane/get_started_view.rs`
- `app/src/coding_panel_enablement_state.rs`
- `app/src/settings_view/code_page.rs`, `app/src/settings_view/code_page_tests.rs`
- `app/src/terminal/view/init_project/mod.rs`, `model.rs`, `lsp_server_selector.rs`
  (whole `init_project/` dir — the project-init flow reached from a terminal tab)
- `app/src/integration_testing/code_review/**` (test harness for the review pane)
- `app/src/integration_testing/goto_line.rs` (drives the code-editor goto-line)

---

## 3. Module/registration edits in Tarp-owned glue files

These are small, surgical edits in `app/`-owned files (safe to diverge):

- **`app/src/lib.rs`**
  - Remove `mod code;` (18), `mod code_review;` (19), `mod coding_entrypoints;`
    (20), `mod coding_panel_enablement_state;` (21).
  - Remove `coding_entrypoints::project_buttons::init(ctx);` call.
  - Remove imports/uses of `GlobalBufferModel`, `GlobalCodeReviewModel`,
    `LanguageServerShutdownManager`, `GitStatusUpdateModel` and their singleton
    registration in the app init path.
- **`app/src/features.rs`** — delete the `#[cfg(feature = "...")] FeatureFlag::X,`
  lines for every flag in §1a/§1b (lines 207, 227, 239, 259, 261, 265, 267, 269,
  281, 297, 301, 303, 321, 441, 477, 495, …).
- **`app/src/settings_view/mod.rs`** — drop the `code_page` module decl and any
  navigation entry that opens the Code settings page.
- **`app/src/integration_testing/mod.rs`** — drop `code_review` + `goto_line` mods.

---

## 4. Partial-edit files (keep the file, prune cluster bits)

These are Tarp-owned files where the cluster is one of several concerns. Edit in
place; do not delete.

- **`app/src/pane_group/pane/mod.rs`** — `IPaneType` enum (136–155). Remove
  variants `Code`, `CodeDiff`, `GetStarted` and every `match`/`Display`/
  constructor arm that mentions them (`from_code_pane_ctx`,
  `from_get_started_pane_ctx`, `from_code_pane_view`, the `IPaneType::Code =>`
  render arms at 459–460 / 487–488, the `is_code()` helper at 421). Remove
  `use crate::code::view::CodeView;` and `use ...get_started_view::GetStartedView;`.
- **`app/src/pane_group/mod.rs`** — remove `Code(...)`, `CodeReview(_)`,
  `GetStarted` `LeafContents` arms (1739–1956 region), the `CodeView`/`CodeSource`/
  comment imports (80–95, 200), and the `pane_group::Event` variants
  `OpenCodeReviewPane`/`ToggleCodeReviewPane`/`InsertCodeReviewComments`/
  `OpenCodeReviewPaneAndScrollToComment`/`ImportAllCodeReviewComments` and the
  `open_code_*` / `source: CodeSource` fields (547–738, 4941–4963).
- **`app/src/app_state.rs`** — `LeafContents` enum (119–176). Remove
  `Code(CodePaneSnapShot)`, `CodeReview(CodeReviewPaneSnapshot)`, `GetStarted`
  variants and the snapshot type defs (`CodePaneSnapShot`, `CodePaneTabSnapshot`,
  `CodeReviewPaneSnapshot`).
- **`app/src/workspace/view.rs`** — the largest consumer (~30 hits). Remove
  `add_get_started_tab` (11663), the `pane_group::Event::OpenCodeReviewPane`/
  `ToggleCodeReviewPane`/`Insert…`/`…ScrollToComment` handlers (15348–16183), the
  `CodeReviewPaneContext` struct (880–893) + helpers (8781–9041), the
  `GlobalCodeReviewModel` use (8520), `TabSettingsChangedEvent::ShowCodeReview*`
  arms (3618–3628), the `HeaderToolbarItemKind::CodeReview` layout branch (5799),
  the `CodeDiffPane, CodePane, CodeReviewPanelArg` imports (276), and the
  tabbed-editor grouping branch (8134).
- **`app/src/workspace/mod.rs`** + `action.rs` + `view_tests.rs` — remove
  `WorkspaceAction::AddGetStartedTab`, `Open*CodeReview*Panel` actions, the
  `[Debug] … plugin` items that reference the code panel, and the
  `GetStartedTab.override_enabled` test guard.
- **`app/src/pane_group/working_directories.rs`** — remove the
  `file_tree_views: HashMap<…, ViewHandle<FileTreeView>>` map +
  `get_file_tree_view` accessor.
- **`app/src/pane_group/pane/view/header/mod.rs`** — remove the
  `FeatureFlag::CodeLaunchModal` tooltip branch.
- **`app/src/settings_view/features/external_editor.rs`** — keep the file; remove
  only the in-app-pane controls: `OpenCodePanelsFileEditor`,
  `PreferTabbedEditorView`, `ToggleTabbedEditorView`, `SetCodePanelsEditor`, and
  the "Group files into single editor pane" toggle. Keep the
  open-in-external-`$EDITOR` behavior.
- **`app/src/util/file/external_editor/{mod,settings,mac,windows,linux}.rs`** —
  keep open-in-external behavior; prune `OpenCodePanelsFileEditor` /
  `PreferTabbedEditorView` enum members and their handling.
- **`app/src/context_chips/display_chip.rs`** + `current_prompt.rs` — remove the
  `CODE_REVIEW_TOOLTIP_TEXT` import, `DiffStats`/`GitRepoStatusModel` chips, and
  the code-review display-chip action (these are AI-prompt context chips; if the
  AI spec removes `context_chips` wholesale, this is moot).
- **`app/src/auth/mod.rs`** — remove `use crate::code::editor_management::
  {CodeEditorStatus, CodeEditorSummary};` and any auth-status code-editor summary.
- **`app/src/notebooks/file/mod.rs`** — remove the
  `local_code_editor::render_remote_disconnected_banner` call and `CodeSource`
  import (notebooks/file viewer reuses the code editor banner).
- **`app/src/quit_warning/…`, `app/src/uri/…`, `app/src/util/…`,
  `app/src/test_util/…`** — single-line `CodeSource`/`code::` imports; prune.
- **`app/src/search/command_palette/files/data_source.rs`,
  `app/src/search/files/icon.rs`** — file-search command-palette entries that open
  code panes; remove the open-in-code-pane path. (The other `search/` hits are
  `ai_context_menu/*` and go with the AI spec.)

### Persistence — TRACKED crate, flag the migration risk
`app/src/persistence/sqlite.rs` (73–87, 857, 1190–1286) and
`app/src/persistence/mod.rs` serialize/restore `CodePaneSnapShot`,
`CodePaneTabSnapshot`, `CodeReviewPaneSnapshot` to SQLite. The **table/schema/
model definitions live in the tracked `crates/persistence` crate**:
- `crates/persistence/src/schema.rs` — `code_panes` (125), `code_pane_tabs`
  (116), `code_review_panes` (133).
- `crates/persistence/src/model.rs` — `CodePane`/`NewCodePane` (433/615),
  `CodePaneTab`/`NewCodePaneTab` (442/623), `CodeReviewPane`/`NewCodeReviewPane`
  (452/631).
- Migrations: `crates/persistence/migrations/2024-05-21-183957_add-code-pane`,
  `…/2025-09-29-154015_add_code_review_pane`,
  `…/2026-04-14-150000_add_code_pane_tabs`.

**Recommendation:** do NOT edit the tracked persistence schema/model/migrations.
Instead, in the Tarp-owned `app/src/persistence/sqlite.rs`, drop the
serialize/restore arms for the removed `LeafContents` variants and treat any
persisted code/code-review panes as "skip on restore". The tables stay (dormant)
to keep `crates/persistence` upstream-shaped and merge-clean. Only if a hard
schema cleanup is wanted later, add a *new* Tarp migration that drops the tables
(append-only; never edit upstream migrations).

---

## 5. Cross-cutting integration points (the real risk)

82 files outside the cluster reference it. Distribution by top dir:
`ai` 19, `pane_group` 12, `terminal` 11, `search` 7, `remote_server` 7,
`workspace` 6, `context_chips` 3, plus singles. Key shared types exported by the
cluster and imported elsewhere:

| Exported type | Defined in | Notable importers |
|---|---|---|
| `CodeSource` | `code/editor_management.rs` | `ai/agent/*`, `app_state.rs`, `pane_group/mod.rs`, `terminal/view*`, `notebooks/file`, `auth` |
| `LocalOrRemotePath` | `code/buffer_location.rs` | `ai/agent_sdk/driver/output.rs`, `pane_group/mod.rs`, `app_state_tests` |
| `CodeEditorView` / `CodeEditorEvent` | `code/editor/view.rs` | 37 files, overwhelmingly `ai/blocklist/**` |
| `ReviewComment*`, `CommentId`, `ReviewCommentBatch`, `AttachedReviewComment*` | `code_review/comments/` | `ai/agent/{comment,mod,redaction,conversation}.rs` |
| `CommentViewCard` | `code_review/comment_rendering.rs` | `ai/blocklist/block.rs` |
| `DiffViewer`, `DisplayMode`, `InlineDiffView` | `code/diff_viewer.rs`, `code/inline_diff.rs` | `ai/blocklist/inline_action/code_diff_view.rs`, `ai/blocklist/block/view_impl/output.rs` |
| `GitRepoStatusModel`, `GitStatusUpdateModel` | `code_review/git_status_update.rs` | `ai/blocklist/context_model.rs`, `context_chips/current_prompt.rs`, `terminal/input_tests.rs` |
| `CodeReviewTelemetryEvent`, `CodeReviewPaneEntrypoint` | `code_review/telemetry_event.rs` | `ai/agent/conversation.rs`, `ai/blocklist/block.rs`, `terminal/view/{action,use_agent_footer}.rs`, `terminal/input/slash_commands` |

**The dominant coupling is AI ↔ Warp Code.** The AI blocklist renders code blocks
and inline diffs with `CodeEditorView`/`DiffViewer`, and the agent comment/redaction
pipeline speaks `ReviewComment*`. This means:

> **Sequencing rule:** remove the **AI cluster first (or jointly)**, then Warp
> Code. If AI is removed first, ~19 `ai/` + several `terminal/` importers vanish,
> shrinking this cluster's external surface from 82 files to roughly the
> `pane_group`/`workspace`/`app_state`/`persistence`/`settings_view` glue in §4 —
> which is purely structural pane plumbing. Attempting Warp-Code removal while AI
> still compiles forces you to stub all those AI imports, which is wasted work.

The `terminal/` references are themselves AI-path code (`cli_agent.rs`,
`use_agent_footer/`, `slash_commands/`, `view/init_project/`), **not** terminal
rendering. They go away with AI + this spec; `crates/warp_terminal` itself is
untouched.

---

## 6. Removal / sequencing order

0. **Prereq:** land (or co-stage) the **AI removal** so `ai/blocklist/**` and
   `ai/agent/**` no longer import `CodeEditorView` / `ReviewComment*` / `CodeSource`.
   (Per [`../05-removal-map.md`] global order, Telemetry → Code-editor; but
   because of the AI↔Code coupling, in practice do AI alongside or just before.)
1. **Flip flags off** — remove the §1a (and assessed §1b) features from `default`
   in `app/Cargo.toml`. Build: the cluster still *compiles* (modules are
   unconditional) but is now runtime-disabled. Smoke-test the app still launches.
2. **Sever the pane plumbing** — edit `pane_group/pane/mod.rs` (`IPaneType`),
   `pane_group/mod.rs` (`LeafContents` arms + `Event` variants),
   `app_state.rs` (`LeafContents` + snapshot types), `workspace/view.rs` +
   `workspace/mod.rs` (handlers/actions). This is the heaviest mechanical step.
3. **Prune persistence (Tarp-owned side only)** — drop the code/code-review
   serialize/restore arms in `app/src/persistence/sqlite.rs`; leave
   `crates/persistence` schema/model/migrations untouched (§4).
4. **Delete the cluster dirs/files** — §2a–2d.
5. **Prune partial-edit consumers** — §4 (settings external_editor, notebooks,
   auth, context_chips, search file entries, working_directories, header tooltip).
6. **Remove module decls + flag registrations** — §3 (`lib.rs`, `features.rs`,
   `settings_view/mod.rs`, `integration_testing/mod.rs`).
7. **Assess and drop the `lsp` crate** — §7.
8. **Rebuild + run** after each chunk (`./script/presubmit` / M1 smoke test).
   Keep `crates/warp_terminal` & `warp_core` compiling at every step.

---

## 7. Crate-level fallout: `crates/lsp`

The `lsp` workspace crate is consumed by **22 app files**, all of which are in the
removal set: the entire `code/**` cluster, `code_review/**`, `ai/persisted_workspace.rs`,
`persistence/{mod,sqlite}.rs`, `workspace/mod.rs`, `settings_view/code_page.rs`,
and `terminal/view/init_project/**`. Once Warp Code + AI are removed, `lsp` has no
consumers → **delete `crates/lsp`** and drop:
- `lsp.workspace = true` (`app/Cargo.toml:144`),
- `"lsp/local_fs"` from the `local_fs` feature (`app/Cargo.toml:764`),
- the `lsp` member from the workspace `Cargo.toml`.

**Do NOT** flip the whole `local_fs` feature (`app/Cargo.toml:762`) for this
spec: it also gates `ai/local_fs`, `persistence/local_fs`, `repo_metadata/local_fs`,
and `warp_core/local_fs`. `warp_core` is tracked terminal-core; touching `local_fs`
risks disabling local-filesystem behavior the terminal needs. Only remove the
`lsp/local_fs` sub-entry. Re-evaluate the full `local_fs` flag in the AI/cloud specs.

---

## 8. Items that touch a TRACKED crate (merge-conflict flags)

The whole point of [`../08-upstream-sync.md`] is to keep tracked crates
upstream-shaped. This cluster's removal is **almost entirely Tarp-owned `app/`
work**, with these tracked-crate exceptions to handle carefully:

1. **`crates/warp_features/src/lib.rs`** — holds the `FeatureFlag` enum variants
   for the cluster (lines 320–873). **Do not delete the variants.** Disable via
   `default`-feature removal + `app/src/features.rs` cfg-line removal (§1c). Zero
   edits to this tracked crate.
2. **`crates/persistence`** (`schema.rs`, `model.rs`, `migrations/*`) — holds the
   `code_panes` / `code_pane_tabs` / `code_review_panes` tables. **Leave as-is**;
   do the removal only in the Tarp-owned `app/src/persistence/sqlite.rs` restore
   path (§4). Optional later: append a new Tarp-only drop migration.
3. **`crates/lsp`** — a removable crate, but its deletion changes the workspace
   `Cargo.toml` member list and `app/Cargo.toml` deps. That's expected divergence,
   not a conflict risk (the crate is wholly owned by the removed surface).

Verified clean: `crates/warp_terminal`, `warp_core`, `editor`, `command`,
`warpui`, `warpui_core` contain **zero** references to the code cluster — no edits
needed there, preserving merge-cleanliness.
