# Wave 0 — Removal Specs (index)

File-level worklists produced by the Wave 0 analysis fan-out, plus the critic's
review. Generated 2026-06-05; read [`00-review.md`](00-review.md) first for the
go/no-go and the cross-spec reconciliations.

## Legend (filename ↔ A-number) — G1 fix

Specs cross-reference each other by A-number; this is the canonical mapping.
**Cite the filename, not the A-number.**

| A# | File | Surface |
|---|---|---|
| A1 | [`ai.md`](ai.md) | AI / agents / MCP / computer-use / voice / classifiers |
| A2 | [`cloud-accounts.md`](cloud-accounts.md) | Cloud / accounts / sharing / oz / secrets / remote-server |
| A3 | [`telemetry.md`](telemetry.md) | Telemetry / analytics / Sentry / crash-reporting |
| A4 | [`code-editor.md`](code-editor.md) | Warp Code cluster (editor pane / file-tree / diff / PR review / LSP) |
| A5 | [`dep-graph.md`](dep-graph.md) | Crate reverse-dep graph + leaf-first deletion order |
| A6 | [`branding-map.md`](branding-map.md) | Warp→Tarp branding, channels, assets, URLs, env vars |
| A7 | [`ci-plan.md`](ci-plan.md) | `.github/` + `script/` CI/release plan + draft workflows |
| A8 | [`feature-flags.md`](feature-flags.md) | All 292 cargo features → keep/remove + minimal `default` |

> "doc 05" = [`../05-removal-map.md`](../05-removal-map.md) (the original removal map);
> not the same as A5 (`dep-graph.md`).

## Locked product decisions

See [`../09-parallel-execution-plan.md`](../09-parallel-execution-plan.md#product-decisions-locked-2026-06-05):
D1 keep Mermaid · D2 remove NL-to-command · D3 remove LSP (verified: no terminal
path consumes `crates/lsp`) · D4 keep autosuggest/autocomplete, remove only the AI
predictor layer.

## Feature-ownership ledger (G2)

For features co-claimed by two specs, this names the **single owner of the code
deletion** (separate from who flips the `default`). Prevents the "each assumes the
other owns it, neither removes it" failure mode.

| Feature(s) | Code-deletion owner | Note |
|---|---|---|
| `cloud_conversations` | A2 (cloud) | Code lives under `app/src/ai`, but it's a cloud surface — A2 owns deletion; A1 just defers. |
| `agent_shared_sessions` | A2 (cloud) | Sharing surface. |
| `global_ai_analytics_collection` / `_banner` | A3 (telemetry) | A1/A8 defer to A3. |
| `command_predictor`, `prompt_suggestions_via_maa`, AI `ai_history:` filter | A1 (ai) | The AI predictor layer (D4). Autosuggestion engine itself stays. |
| `pr_comments_*`, `diff_set_as_context`, `ai_context_menu_code`, `selection_as_context`, `github_pr_prompt_chip` | A4 (code-editor) | PR/diff-as-context cluster; A8 just flips defaults. |
| `search_codebase_ui`, `command_palette_file_search` (code-pane arm) | A4 (code-editor) | A4 prunes the code-pane-opening path; the plain file-search feature survives (A8 keeps `command_palette_file_search`). |
| `markdown_mermaid` + `mermaid-to-svg` dep | — (KEEP, D1) | No deletion owner; stays in default. |
| `graphql` / `warp_graphql_schema` (+ `default-members` edit) | A5 (dep-graph) | A2 defers, A5 owns; deletion needs a `Cargo.toml:default-members` edit. |
| `input_classifier` / `natural_language_detection` | A1 (ai) | Removed per D2 (NL-to-command). |
| `crates/lsp` (+ `node_runtime`/`warp_js` if unused) | A4 (code-editor) | Removed per D3; verified no terminal consumer. |

## Ordering & cross-refs (from the critic)

- **Global app-layer order:** adopt A4's **"AI before/with code-editor"** rule (the
  `ai/blocklist/**` modules import `CodeEditorView`/`ReviewComment*`), which
  supersedes doc 05's literal "code-editor first" ordering.
- **Cloud cluster deletes as one atomic batch** (A5 found a `cloud_objects ↔
  warp_server_auth` cycle) — not a strict linear sequence.
- **`RELEASE_FLAGS` cross-ref:** release bundles force-enable `CrashReporting` via
  `app/src/features.rs` (the Tarp-owned bridge). A3 (telemetry) must handle this
  there — do **not** edit the tracked `warp_features` enum. See A8 §5/§8.

## Per-spec verification gate

Each Wave-2 step ends with the M1 smoke test: `cargo build --bin warp-oss
--features gui` + `./script/run --dont-open` launches. A2 adds a behavioral gate
(runs logged-out, dials no `*.warp.dev`); A3 adds a "prove telemetry is gone" grep.
