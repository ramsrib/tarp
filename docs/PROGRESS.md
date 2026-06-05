# Tarp — Work Log

Chronological record of work done on the fork. Newest entries at the top.
Companion to [`../TARP-PLAN.md`](../TARP-PLAN.md) (the plan) and
[`README.md`](README.md) (the audit). Build artifacts live in `/tmp/tarp-build/`.

---

## 2026-06-05

### Wave 2 — step 3: AI-removal pass → `voice_input` removed; AI is monolithic  ✅/finding (branch `dewarp`, `c00d58e3`)
- Ran a worktree-isolated agent to attempt the AI/agent removal. Outcome: it
  removed the one cleanly-excisable AI crate (**`voice_input`**, see
  [`REMOVED.md`](REMOVED.md)) to a green build with **zero tracked-crate edits**, and
  then correctly stopped.
- **Major scoping finding:** the rest of the AI surface is **one monolithic,
  all-at-once removal** — no further clean leaves:
  - `ai`, `mcp`, `computer_use`, `input_classifier`, `natural_language_detection`
    are **unconditionally compiled** (not feature-gated); `mcp`/`computer_use` are
    leaves *of `ai`*, not independently removable.
  - `mod ai;` is unconditional; AI singletons are added unconditionally in
    `lib.rs`; **`AISettings` has 1,146 refs across 114 always-compiled files**,
    incl. `terminal/view.rs` (**28,306 lines**), `workspace/view.rs`,
    `terminal/input.rs`, persistence, server. It also reaches into the sibling
    cloud / code-editor / notebooks surfaces.
  - ⇒ removing AI = untangling `lib.rs`'s singleton graph + the central terminal
    files in one sustained ~629-file / 222k-LOC effort; it cannot be sliced into
    green-building increments.
- **Revised plan:** the AI removal is a dedicated, sustained pass (its own branch,
  many build iterations, the approved `persistence` edit), not incremental leaves.
  Decide vehicle/approach before starting. Until then `dewarp` stays green with the
  reduced default + `voice_input` gone.
- Left ~333 inert `#[cfg(feature="voice_input")]` warning-only sites to be deleted
  with the AI pass (voice is AI-coupled) — not polished now (throwaway churn).

### Wave 2 — step 2: telemetry no-op baseline + entanglement findings  ✅ (analysis/correction)
- **Telemetry Phase 1 (no-op) confirmed as the achieved state.** OSS channel
  defaults to `telemetry_config: None` / `crash_reporting_config: None`; step-1
  build (`--features gui`, excludes `crash_reporting`/`cocoa_sentry`) launched with
  telemetry inert and dialed nothing. No code/tracked edits needed for "off".
- **Key correction to the A3 spec:** Group A ("delete `app/src/server/telemetry/`
  wholesale, keep the 774 calls compiling") is NOT standalone-achievable. The
  `TelemetryEvent` enum lives in `events.rs:1262` and is referenced via
  `TelemetryEvent::Variant` at 157+ sites across `app/src`. Telemetry code deletion
  therefore **rides with the feature removals** (AI/cloud/sharing/onboarding/
  code-editor) that own those call sites, not as an up-front cut. Spec updated.
- **General finding:** removals are more entangled than the specs implied. E.g.
  `voice_input` (already unwired from `gui`) still has 41 refs woven into the app
  input-editor view (`app/src/editor/view/voice.rs` + state/rendering). Each
  removal cascades; the big **AI removal** in particular is a large iterative
  effort, best done in a focused worktree-isolated pass (per `docs/09` Wave 2),
  not a quick deletion.
- **Revised approach:** keep the feature-flag-reduced binary (step 1) as the stable
  base; execute the heavy code/crate deletions as dedicated, individually-verified
  passes (AI first/with code-editor, then cloud/accounts), each ending in the M1
  smoke test + a commit.

### Wave 2 — step 1: minimal default feature set  ✅ (branch `dewarp`)
- Created branch **`dewarp`** off `master` to keep `master` pristine (it doubles as
  the future upstream-sync mirror).
- **LSP audit (D3):** confirmed every consumer of `crates/lsp` is in a removed
  surface (`app/src/code/**`, `code_review/**`, `ai/**`, `settings_view/code_page.rs`,
  `terminal/view/init_project/**`). No terminal-only path uses LSP → safe to delete.
- **Applied A8's minimal `default`:** `app/Cargo.toml` 187 → **49** features
  (validated all 49 are defined). Original 189-line block saved to
  `/tmp/tarp-build/original_default.txt`. Rewired `gui = ["voice_input"]` → `gui = []`.
- **Verified:** `cargo build --bin warp-oss --features gui` → exit 0, 0 errors,
  47 benign warnings (dead-code from now-off features). Bundled `WarpOss.app`
  (codesigned, exit 0). Launched → window + Metal renderer up, quit clean.
- Note: AI/cloud subsystems + `*.warp.dev` URLs still appear at runtime — expected;
  crates are still linked. This step only shrinks default *behaviors* (reversible).
  Their removal is later Wave 2 steps.
- **Critic reconciliations applied** (from `docs/removal/00-review.md`): C1
  (persistence forced-edit note in `dep-graph.md`), G1 (filename↔A# legend), G2
  (feature-ownership ledger) — both in new `docs/removal/README.md`; A4 "AI
  before/with code-editor" ordering; A3↔A8 `RELEASE_FLAGS` cross-ref.

### Wave 0 — de-Warp analysis fan-out  ✅
- Ran an 8-agent parallel workflow (`wf_b0d4f86a-b9b`, 9 agents incl. critic,
  ~736k tokens, ~4.5 min) → `docs/removal/{ai,cloud-accounts,telemetry,code-editor,
  dep-graph,branding-map,ci-plan,feature-flags}.md` + `00-review.md`.
- Critic verdict: **conditionally ready for Wave 2**; keystone artifact = A8's
  187→49 minimal `default`.
- **Locked product decisions** (`docs/09` §Product decisions): D1 keep Mermaid ·
  D2 remove NL-to-command (`input_classifier`/`natural_language_detection`) ·
  D3 remove LSP · D4 keep autosuggest/autocomplete, remove only the AI predictor
  layer (`command_predictor`, `app/src/ai/predict/**`, `prompt_suggestions_via_maa`).

### M1 — green baseline  ✅
- Verified the unmodified fork builds + bundles + launches with no code changes
  (`warp-oss`, 721 MB; `WarpOss.app` launches, Metal renderer, logged-out).
- Required installing the Metal Toolchain (`xcodebuild -runFirstLaunch` then
  `-downloadComponent MetalToolchain`) and the pinned `cargo-bundle`. Documented in
  [`../BUILD.md`](../BUILD.md).

### Audit + planning  ✅
- Full audit captured in `docs/01`..`09` + `TARP-PLAN.md`.
- Settled naming/trademark posture (keep "Tarp", metaphor-only public rationale,
  affiliation disclaimer, distinct branding) — `docs/04`.
- Upstream-sync strategy (selective cherry-pick of terminal-core only; tracked-vs-
  owned path split governs how de-Warp is done) — `docs/08`.

### Next
- Wave 2 step 2: telemetry no-op baseline (A3 Phase 1, zero tracked-crate edits).
- Then deletions in locked order: AI (predictor/NL/MCP/computer-use/voice) →
  code-editor + LSP → cloud/accounts.
