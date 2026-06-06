# Tarp — Work Log

Chronological record of work done on the fork. Newest entries at the top.
Companion to [`../TARP-PLAN.md`](../TARP-PLAN.md) (the plan) and
[`README.md`](README.md) (the audit). Build artifacts live in `/tmp/tarp-build/`.

---

## 2026-06-05

### Full Warp→Tarp visible-string audit + identity  ✅ (merged `e4404b03`, `60795c71`)
- **Visible-label sweep (agent):** 151 files, ~466 user-visible "Warp" strings →
  "Tarp" across window titles, settings UI (incl. Input-type radio), menus,
  notifications/banners/tooltips, AI/assistant text, onboarding/dialogs, plugin
  prompts, SSH/Tarpify ("Warpify"→"Tarpify"). Build green, no tracked-crate edits.
- **Identity / paths:** `oss.rs` AppId→dev/tarp/Tarp, logfile `warp-oss.log`→
  `tarp.log`, embedded plist→Tarp (`8f404a14`); config dir `~/.warp-oss`→`~/.tarp`,
  data dir via AppId; `TERM_PROGRAM`→`TarpTerminal` + XTVERSION self-id `Tarp(ver)`
  (`60795c71`).
- **Kept intentionally (not user-visible / functional):** `WARP_*` shell-integration
  env vars (terminal↔shell contract, ~38 names/~87 read-sites — renaming = high
  risk, zero branding value); Rust identifiers/types/feature-flag names/crate names;
  `warpdotdev`/`docs.warp.dev` upstream URLs; `warp-oss` artifact names in
  `autoupdate/*` (must match release artifacts; autoupdate is disabled anyway);
  telemetry event payload strings (backend, not UI); the required Denver copyright.
- **Open decisions (agent-flagged):** "Tarpify" verb (applied — confirm); "Warp's
  Terms of Service" link still points at Warp's ToS (Tarp has none — recommend
  removing the link); telemetry payload rebrand (optional, not UI).

### UI sweep 2: input footer + palette + menus + vim banner  ✅ (merged `2bb007b2`)
Second worktree-agent pass (`ff7e6951`..`2bb007b2`), build green, command input intact:
- **Input bar** (the real one = `terminal/universal_developer_input.rs`, NOT the
  agent_input_footer the prior agent checked): removed the **Agent Mode `A` toggle**,
  the **AI model selector** ("auto (cost-efficient)"), the `@` AI-context and `/`
  buttons. Kept `>_` terminal indicator + `+` file-attach. Editor/PTY paths untouched.
- **Command palette**: dropped Warp Drive / Notebooks / Environment Variables /
  Agent workflows / AI Conversations chips; kept Workflows / Files (local) / Actions
  / Sessions / Launch configs.
- **Context menus**: removed "Share session" (the sign-up popup) + "Ask Warp AI"
  across all 4 menu builders.
- **Workflow editor**: removed "Generate title… with Warp AI".
- **Vim banner**: no longer auto-shown on launch (feature kept; enable via settings).
- Also earlier: About Warp wordmark dropped; Privacy + Shared-blocks settings tabs
  removed (`886d3a17`).
- **Residue still compiled-in (not surfaced; later cleanup):** the alternate
  `ai/blocklist/agent_view/agent_input_footer` path (used only if UDI is off /
  CLI-agent sessions), "Save as workflow" drive item (gated on cloud), Ask-AI/Share
  keybinding *registrations* (gated, not visible), and `ai`/`billing`/`pricing`/
  `drive`/`voice` modules. Tracked for the deeper M6 deletion.

### UI sweep: strip AI/account/cloud chrome  ✅ (merged `cd9a8c66`)
Worktree-agent pass (5 area commits `792254c7`..`cd9a8c66`), build green, merged:
- **Settings tabs**: nav now terminal-only (Appearance, Features, Keyboard, Shared
  blocks, Privacy, About) — removed Account/Agents/Cloud/Teams/Referrals/Warpify/
  Drive; default section → Appearance.
- **About**: Tarp wordmark + fork copyright (logo SVG still Warp — asset TODO).
- **Privacy + links** (`util/links.rs`): warp.dev → the Tarp repo.
- **Header**: removed Warp AI, Code review, Warp Essentials toolbar/tab-bar items.
- **Sidebar**: left panel = Global Search only (dropped Project Explorer, Warp
  Drive, conversation list).
- **Footer/input AI** (mode switcher, model selector, A/@): **deferred** — verified
  gated on `agent_view`/`agent_mode` (not in default) so inert/not-rendered when
  logged-out; excising the residual code risks the command input. Left as-is.
- Earlier same session: tab-bar Sign up button, Login-for-AI banner removed; vim
  prompt "Warp's"→"Tarp's" (`55e79cd8`).
- **Remaining AI residue (follow-up, task #20):** "Ask Warp AI" context-menu items
  (`terminal/view.rs`, `init.rs`), workflow "Generate title with Warp AI"
  (`workflow_view.rs`), the Warp wordmark SVG, and dropping construction of the
  now-hidden settings pages + input-footer code residue.

### Test bug: account "Welcome/Sign up" modal on launch  ✅ fixed (`53d4d778`)
- First run showed Warp's account onboarding modal (Sign up / Sign in / Skip).
  Cause: `RootView::new` fell through to `AuthOnboardingState::Auth` because
  `skip_firebase_anonymous_user` isn't in Tarp's minimal default set.
- **Fix:** desktop non-logged-in path now always enters the terminal directly
  (dropped the ForceLogin / agent-onboarding / login branches). Tarp has no accounts.
- Verified: boots straight into the terminal, no modal, 0 panics.

### Test install + first real bug fix  ✅ (`c687f620`)
- Installed Tarp.app to `/Applications` for testing. First launch **crashed on
  foreground** (debug build): `app_menus.rs` built menu items for disabled-feature
  actions (e.g. `ToggleConversationListView`) whose descriptions are absent →
  upstream `debug_assert!` in `default_name()` panicked (release would not, it
  compiles the assert out).
- **Fix:** `default_name()` no longer panics; dropped the whole **AI** and **Drive**
  (cloud) menus from the menu bar (removed features) + stray View-menu items
  (`ToggleWarpDrive`, `ToggleConversationListView`). Verified Tarp.app launches and
  stays up with 0 panics.
- This is the expected class of "non-cfg-gated edge surfaced by disabling features"
  (A8 warned of it). More may surface on deeper interaction; the non-panic fallback
  means they degrade gracefully (a stray "<NO DESCRIPTION>" item at worst) instead
  of crashing — mop up as found.
- Note: also kicked off then **killed** a background release build (it saturated CPU
  and starved the debug app — that was the rustc spike). Build release only when
  idle, or accept the debug build for testing.

### M3 — app icon replaced (release blocker cleared)  ✅ (`f395c1cc`)
- User supplied two candidates (`~/Desktop/tarp-{1,2}.png`); chose **tarp-2** (tarp
  peeled back to reveal a green terminal) — better concept fit, contrast, and
  fills the rounded-square per macOS convention.
- Processed with ImageMagick: corner-floodfill white→transparent (preserves
  interior whites), squared, full-bleed rounded. Replaced
  `channels/oss/icon/no-padding/{512x512.png,icon.ico}`; kept `AppIcon-source.png`
  (high-res transparent master) for regen. Verified `Tarp.icns` bakes into Tarp.app.
- **Icon release blocker: RESOLVED.** Remaining icon polish (non-blocking): retina
  1024/`.icon` adaptive bundle (cargo-bundle's icns generator rejected a >2-entry
  list; current 512 source upscales for retina) and Linux icon install verification.

### M0 — repo hygiene (partial)  ✅ in progress (branch `dewarp`)
- `a0c614c3` — rewrote `README.md` for Tarp (dropped Warp's Oz/cloud/careers/Slack
  sections + product imagery; added affiliation disclaimer, Tarp build commands,
  AGPL/MIT + fork attribution); added `NOTICE` (dual copyright per ADR-002).
- `7b437e9e` — de-Warped `SECURITY.md` + `CODE_OF_CONDUCT.md` contacts (point at the
  Tarp repo / GitHub private advisory instead of warp.dev addresses).
- **M0 (tail) — done:**
  - `506f6ba9` — pruned **18** Warp-internal workflows + CUT actions (docubot,
    get_channel_config) + scripts/issue-triage/STAKEHOLDERS; replaced the 858-line
    `ci.yml` with a minimal fmt-check + linux-build CI.
  - `5c0cd5eb` — de-Warped `dependabot.yml` (dropped warpdotdev assignees/private
    registry/namespace group) + fresh Tarp issue/PR templates (cut AI/Linear/SSH
    /cherrypick/feature-flag variants).
  - `<this>` — rewrote `CONTRIBUTING.md` for Tarp (scope, build→BUILD.md, workflow,
    upstream-sync caution, licensing).
- **M0 — branch rename DONE** (separate work): `main` is default on `origin`,
  `master` deleted, `fork-base` tag + `upstream` remote set (ADR-006).
- **M0 still open:** `WARP.md` (12 KB Warp engineering guide) + `FAQ.md` are still
  Warp-branded — rewrite/retire later; bulk user-facing "Warp" strings (mostly in
  disabled code).

### M3 — branding (Warp → Tarp), identity rename  ✅ verified (branch `dewarp`)
Decisions used (autonomous, per ADR-001): namespace `dev.tarp.Tarp`; keep the
channel enum, rename only the Oss arm (low divergence per ADR-003); placeholder
icon (real art deferred). Savepoints, each committed:
- `b2df3773` SP1 — bundle metadata: `dev.warp.WarpOss`→`dev.tarp.Tarp`, Tarp
  copyright (Denver kept per AGPL), plain-terminal description.
- `ca9b7d77` SP2 — `warp_core` Oss arm runtime identity: `AppId(dev/tarp/Tarp)`,
  url scheme `tarp` (tracked-crate edit, Oss arm only).
- `104ea11c` SP3 — binary `warp-oss`→`tarp`, app `WarpOss.app`→`Tarp.app`
  (`default-run`, `[[bin]]`, bundle metadata key) + all platform bundle/run scripts.
- `9c40fc96` SP4 — `.desktop`→`dev.tarp.Tarp.desktop`, `about.hbs`→Tarp.
- **Verified on macOS:** builds green, bundles **Tarp.app** (`CFBundleIdentifier
  dev.tarp.Tarp`, name Tarp), launches as **Tarp** (process `…/MacOS/tarp` +
  terminal-server child), quits clean.

**M3 remaining / TODO:**
- ⚠ **Icon is still Warp's art** (`app/channels/oss/icon/…`) — needs real Tarp logo
  artwork (can't generate). **Must replace before public release (trademark).**
- Log file still named `warp-oss.log` (derived, not from channel `logfile_name`;
  source not the channel config). Minor; trace + rename to `tarp.log` later.
- User-facing "Warp" strings (window title, About, menus) + `warp.dev` help URLs +
  `authors = ["Warp Team <dev@warp.dev>"]` — large surface (~185 files, much in
  disabled AI/cloud code). Do the visible ones; bulk can follow.

### Strategy decision: disable-first, ship early (ADR-005)  ✅ decided
- **Decided (user):** reach the no-AI/no-cloud terminal for v1 by **disabling**
  (minimal default feature set), not deleting AI/cloud source. Pivot to **branding
  (M3) + first release (M4)** on the current `dewarp` base. Full source deletion is
  an optional later project (M6).
- **Reasoning recorded** in [`DECISIONS.md`](DECISIONS.md) ADR-005: the disabled
  state already meets the user-facing goal; deletion is a ~600–700-file coordinated
  AI+cloud surgery with high upstream divergence (fights ADR-003 sync); ship-early
  matches the release cadence. Removed items stay restorable via REMOVED.md.
- **Next:** M3 — Warp→Tarp branding (bundle ids, app name, icons, strings, shell
  integration) per [`removal/branding-map.md`](removal/branding-map.md).

### Wave 2 — step 4: AI engine-removal feasibility investigation  ✅ finding
- Investigated the "engine-only" middle path the user picked. **Verdict: it doesn't
  exist as a smaller option.** The `ai::` refs are `crate::ai::` (app's internal
  `app/src/ai/`, 459 files / ~222k LOC); **334 external files** import its types;
  `AISettings` itself imports `crate::ai`; and AI is co-entangled with
  cloud/server/auth. So AI removal = one ~600–700-file coordinated AI+cloud surgery,
  not sliceable, high upstream divergence. Detail:
  [`removal/ai-removal-feasibility.md`](removal/ai-removal-feasibility.md).
- **Reframe:** end-state is a spectrum — (a) features off / code present (where we
  are; low effort, low divergence, ships now) vs (b) code deleted (clean but big +
  high divergence). Recommendation: ship the disabled-AI terminal as an early
  release (after branding), treat full deletion as a separate optional large project.

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
