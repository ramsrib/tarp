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

### UI surfaces — AI/account/cloud chrome removed (2026-06-05)
App-layer UI removals (render code gated/removed); underlying crates still compiled
in (full source deletion = later M6). Each commit is revertable. Surfaced during
interactive testing; see [`PROGRESS.md`](PROGRESS.md) for the narrative.

| Surface | What was removed | Commit |
|---|---|---|
| App menu bar | "AI" + "Drive" menus; Warp Drive / Conversation View items; disabled-action menu panic fix | `c687f620` |
| Startup | Account "Welcome / Sign up" modal — boots straight to terminal | `53d4d778` |
| Tab bar / banner | Anonymous "Sign up" button; "Login for AI" banner; vim prompt "Warp's"→"Tarp's" | `55e79cd8` |
| Settings tabs | Account, Agents, Cloud platform, Teams, Referrals, Warpify, Warp Drive, Privacy, Shared blocks (kept Appearance/Features/Keyboard/About) | `792254c7`, `886d3a17` |
| About page | Warp wordmark SVG dropped; Tarp text + fork copyright | `202a6753`, `886d3a17` |
| Privacy/links | `warp.dev` → Tarp repo (`util/links.rs`) | `2708faf7` |
| Header toolbar | Warp AI, Code review panel, Warp Essentials + AI/cloud icons | `3ef739f0` |
| Left sidebar | Project Explorer, Warp Drive, Conversation list; then Global Search + the sidebar toggle | `cd9a8c66`, `45c5fe89` |
| Input toolbar row | Agent Mode toggle, AI model selector ("auto (cost-efficient)"), `@` context, `/` slash, `+` AI file-attach, `>_` dead mode indicator | `ff7e6951`, `3b7ede33` |
| Command palette | Warp Drive, Notebooks, Environment Variables, Conversations chips (kept Workflows/Files/Actions/Sessions/Launch configs) | `9dbb6996` |
| Context menus | "Share session" (sign-up popup), "Ask Warp AI" (all builders) | `73a01f60` |
| Workflow editor | "Generate title… with Warp AI" | `953a8d19` |
| Vim banner | No longer auto-shown on launch (feature kept; enable via settings) | `2bb007b2` |
| Debug build noise | Per-block memory-stats footer defaulted off; "(nld overridden)" prompt tag dropped | `093f6ba8` |
| Native menu bar | App menu: About→Tarp, Set-Default→Tarp, title→Tarp; removed Toggle Resource Center, Invite People, Privacy Policy, Log out. Edit "Use Tarp's Prompt". Help: only GitHub Issues (Tarp repo). File: removed New Agent Tab. Tab right-click: removed Share session/Stop sharing | `1a889c4b` |

> **Tracked-crate edit (logged for upstream-sync):** `crates/warpui/src/platform/mac/menus.rs` — "Hide Warp"/"Quit Warp" → "Hide Tarp"/"Quit Tarp" (hardcoded app-name in the standard menu items; justified branding). Only this one warpui string edit so far.

> **Compliance — do NOT remove:** the About page "Portions © 2020-2026 Denver
> Technologies, Inc." line is **required** by AGPL/MIT (upstream copyright notice).
> It is correctly worded as a fork ("Portions ©" alongside "© The Tarp Authors").

**Restore any row:** `git revert <commit>`.
**Kept (decided terminal features):** working-directory picker, vim keybindings (via
settings), command palette + Files search, workflows, themes, SSH, tabs/panes.
**Still compiled-in but not surfaced (later M6 source deletion):** `ai`/`billing`/
`pricing`/`drive`/`voice` modules; the alternate `agent_input_footer` path; the
cloud-gated "Save as workflow" item; Ask-AI/Share keybinding registrations (gated).

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
