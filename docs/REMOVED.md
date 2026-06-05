# Removed Features Registry

A running record of what the de-Warp work removes or disables, so anything can be
brought back deliberately. Two restore mechanisms back this:

1. **Git** — every removal is its own commit on the `dewarp` branch. Restore a
   removal with `git revert <commit>`, or recover specific files with
   `git checkout <commit>^ -- <path>`. The pre-fork baseline is `master` (also the
   upstream-sync mirror).
2. **This registry** — a human-readable index mapping each feature/area to *what*
   was removed and the *exact commit* that did it.

Two categories:
- **Disabled** — code still present, just off (e.g. removed from the `default`
  feature set). Restore = re-enable the feature; no code is gone.
- **Deleted** — code/crates removed. Restore = `git revert` the removal commit.

---

## Disabled (feature-flag, code still present)

### Default feature set reduced 187 → 49 — commit `f79a0001`
- **What:** `app/Cargo.toml` `default` array trimmed from 187 to 49 terminal-only
  features; `gui = ["voice_input"]` → `gui = []`.
- **Effect:** ~138 AI/cloud/agent/code-editor/telemetry features are no longer on
  by default. The code for them is **still present and compilable** (each is still
  a defined feature) — this only changes what's on out of the box.
- **Full list of disabled features:** the original 187-entry block is saved at
  `/tmp/tarp-build/original_default.txt`; the diff is in commit `f79a0001`
  (`git show f79a0001 -- app/Cargo.toml`). Categorized keep/remove rationale:
  [`removal/feature-flags.md`](removal/feature-flags.md).
- **Restore (all):** `git revert f79a0001` (also reverts docs) or re-add specific
  features to the `default = [ ... ]` array in `app/Cargo.toml`.
- **Restore (one feature):** add its name back to `default` — e.g. to re-enable
  agent mode, add `"agent_mode"` (and its dependents) to the array.

---

## Deleted (code/crates removed)

### `voice_input` crate (AI dictation) — commit `c00d58e3`
- **Removed crates:** `crates/voice_input` (422 LOC) — the AI voice-dictation engine.
- **Removed features:** `voice_input` (`app/Cargo.toml`); `gui` no longer pulls it
  (already `gui = []` from step 1).
- **Removed deps:** `voice_input` from workspace `Cargo.toml [workspace.dependencies]`
  and from `app/Cargo.toml`; `Cargo.lock` regenerated (−206 lines).
- **Tracked-crate edits:** none.
- **Build:** green (`cargo build --bin warp-oss --features gui`, 0 errors).
- **Left inert (deferred to the AI pass):** ~333 `#[cfg(feature = "voice_input")]`
  sites across `app/src` (e.g. `terminal/view.rs`, `root_view.rs`, `lib.rs`,
  `editor/view/*`) now reference a removed feature → permanently-off dead cfg
  (compile-skipped) that emits `unexpected_cfgs` warnings only. Also left on disk:
  `app/src/editor/view/voice.rs` and `app/src/voice/` (the latter is the server-side
  `VoiceTranscriber`, not the crate). These are deleted as part of the AI removal,
  since voice is coupled to the AI stack (`ai::blocklist`, `server_api::TranscribeError`).
- **Restore:** `git revert c00d58e3` (brings back the crate, deps, and lock).
- **Why removable now:** it was the only AI crate already fully feature-gated off
  (absent from the default+gui compiled target / `cargo tree`), so it excised to a
  green build without touching the AI core.

<!-- Template for each deletion:
### <feature/area> — commit `<sha>`
- **Removed crates:** crates/x, crates/y
- **Removed app files/dirs:** app/src/...
- **Removed features:** feat_a, feat_b
- **Removed deps:** dep = "..." (Cargo.toml)
- **Tracked-crate edits (merge-conflict risk):** crates/... — <what>
- **Restore:** `git revert <sha>` (or `git checkout <sha>^ -- <paths>` for a subset)
- **Notes:** <coupling, gotchas>
-->
