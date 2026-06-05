# Tarp — Work Log

Chronological record of work done on the fork. Newest entries at the top.
Companion to [`../TARP-PLAN.md`](../TARP-PLAN.md) (the plan) and
[`README.md`](README.md) (the audit). Build artifacts live in `/tmp/tarp-build/`.

---

## 2026-06-05

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
