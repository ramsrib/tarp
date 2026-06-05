# AI Removal Feasibility — investigation finding (2026-06-05)

Investigated whether `crates/ai` + the AI app-module can be excised via a smaller
"engine-only" path (leaving `AISettings` inert) instead of the full surgery.

## Verdict: no cheap middle path. AI removal = full coordinated AI(+cloud) surgery.

### Evidence
- The pervasive `ai::` references in app code are **`crate::ai::`** — the app's
  internal **`app/src/ai/` module (459 files, ~222k LOC)**, not just the
  `crates/ai` library.
- **334 files outside `app/src/ai/`** import `crate::ai::` types (2,593 lines) —
  spread across `server/`, `auth/`, `workspace/`, `editor/`, `tab`, `env_vars`,
  `undo_close`, settings, tests. Removing the AI module forces edits in all of them.
- **`AISettings` is not separable:** its definition (`app/src/settings/ai.rs`,
  1,989 lines) itself imports `crate::ai::request_usage_model::RequestLimitInfo`.
  So "leave settings inert, drop the engine" doesn't hold — settings depends on the
  module too.
- **AI is co-entangled with cloud/server/auth:** 13 `app/src/server/` files and
  3 `app/src/auth/` files use `crate::ai::` types; `app/src/ai/` reaches into
  `server/cloud_objects`, `sync_queue`, `server_api`. AI and cloud cannot be cleanly
  separated — they must be removed together or with heavy mutual stubbing.
- `crates/ai`, `mcp`, `computer_use` are unconditionally compiled; `mcp`/`computer_use`
  are leaves *of `ai`*. `voice_input` (already removed) was the only feature-gated leaf.

### Consequence
A source-level AI removal is a **single sustained ~600–700-file effort that must
also handle cloud/server/auth coupling**. It cannot be sliced into green-building
increments (the `lib.rs` singleton graph + `crate::ai` type imports break compilation
until the whole web is resolved). High effort, high risk, and **high upstream
divergence** (hurts the selective-sync strategy in `docs/08`).

## Implication: "disable" may beat "delete"

The current `dewarp` state already achieves the **user-facing** goal — a terminal
with **no AI/cloud features active** (all cfg-gated off in the OSS/minimal-default
build; OSS dials no network, runs logged-out). What "delete" adds over "disable":
smaller binary, AI source not shipped — at the cost of the large surgery above and
much harder upstream syncing.

Per `docs/08-upstream-sync.md`, **keeping the AI/cloud code present-but-disabled is
the low-divergence choice** and keeps cherry-picking terminal fixes easy. Full
deletion maximizes divergence.

### Recommended sequencing
1. **Ship the disabled-AI terminal as an early release** (after branding M3): it
   meets the no-AI/no-cloud *experience* goal now and matches the "burst of releases
   early" cadence.
2. Treat **full AI+cloud source deletion as a separate, optional, large project** —
   decide later whether the binary-size/source-purity win is worth the surgery +
   divergence cost. If pursued, do it as one coordinated AI+cloud pass on its own
   branch, leaning hard on the compiler, accepting a long red period.

This reframes the de-Warp end state as a spectrum:
- **(a) features off, code present** ← we are here (low effort, low divergence, ships now)
- **(b) code deleted** ← clean source/smaller binary, high effort, high divergence
