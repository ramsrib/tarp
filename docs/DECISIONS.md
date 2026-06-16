# Tarp — Decision Log

Canonical record of the consequential decisions on the fork and *why*, so future
work (and future maintainers) don't relitigate settled questions. ADR-style,
newest at the bottom. Detail lives in the linked docs; this is the index + rationale.

---

## ADR-001 — Keep the name "Tarp"; brand fully distinct
**Decision:** Ship as "Tarp" with a distinct logo/visual identity; the published
rationale is the tarp metaphor only.
**Why:** Memorable, short. The copyright axis is safe (AGPL irrevocable while we
comply); residual risk is trademark, mitigated by full de-branding, an affiliation
disclaimer, nominative-fair-use ("a fork of Warp") only, and a clearance search
before first release. Lock the brand before first public release.
**Detail:** [`04-licensing.md`](04-licensing.md#trademark-separate-from-the-code-license).

## ADR-002 — Stay open-source under AGPL-3.0; do not attempt to relicense
**Decision:** Keep the inherited AGPL-3.0 (+ MIT for `warpui*`); add a Tarp
copyright alongside the upstream one; never remove upstream notices.
**Why:** We are licensees, not the copyright holder — we cannot relicense Warp's
code. AGPL is irrevocable while we comply, so they cannot force a takedown of
compliant code. Removing cloud/network features makes the §13 network clause moot
for a plain local terminal.
**Detail:** [`04-licensing.md`](04-licensing.md).

## ADR-003 — Selective upstream sync; concentrate divergence
**Decision:** Pull upstream fixes by **cherry-pick / path-scoped** of terminal-core
paths only — never a full merge. Maintain a tracked-vs-owned path split; keep
terminal-core crates close to upstream. `master` is the pristine upstream mirror.
**Why:** Upstream runs ~14 commits/day, mostly AI/cloud (irrelevant to us). Keeping
divergence concentrated in the app/AI/cloud layer keeps the worthwhile fixes
low-conflict. Don't depend on upstream — treat sync as a bonus.
**Detail:** [`08-upstream-sync.md`](08-upstream-sync.md).

## ADR-004 — Product scope of features (D1–D4)
**Decisions:** D1 keep Mermaid rendering · D2 remove NL-to-command
(`input_classifier`/`natural_language_detection`) · D3 remove LSP (powered the code
editor; verified no terminal path uses it) · D4 keep the autosuggestion/autocomplete
engine, remove only the AI predictor layer (`command_predictor`,
`app/src/ai/predict/**`, `prompt_suggestions_via_maa`).
**Why:** Keep genuinely-terminal features; drop AI-adjacent ones. The autocomplete
engine (history/completions/specs/corrections) is non-AI and core.
**Detail:** [`09-parallel-execution-plan.md`](09-parallel-execution-plan.md#product-decisions-locked-2026-06-05).

## ADR-005 — De-Warp by DISABLE first, not DELETE; ship an early release on the disabled base
**Decision (2026-06-05):** For the first release, achieve the no-AI/no-cloud
terminal by **disabling** those surfaces (the minimal default feature set, OSS
channel) rather than deleting their source. Pivot now to **branding (M3) + first
release (M4)** on the current `dewarp` base. Treat full source deletion of AI/cloud
as a **separate, optional, later project**, pursued only if binary-size / source
purity justifies the cost.

**Why:**
- **The disabled state already meets the user-facing goal.** The current `dewarp`
  build is a working terminal with AI/cloud/accounts/editor features off (cfg-gated;
  OSS dials no network, runs logged-out). "Delete" only adds smaller binary +
  source purity — not a different user experience.
- **Source deletion is disproportionately expensive and risky.** Investigation
  showed no incremental/clean path: `app/src/ai/` is 459 files / ~222k LOC, imported
  by **334 external files**; `AISettings` itself depends on it; AI is co-entangled
  with cloud/server/auth. It's one ~600–700-file coordinated AI+cloud surgery that
  can't be kept green partway through.
- **Deletion fights our own sync strategy (ADR-003).** Full deletion maximizes
  divergence from upstream, making the selective cherry-pick of terminal fixes much
  harder. Disable = low divergence = sustainable.
- **Matches the release philosophy.** A publishable Tarp now (burst of releases
  early → stabilize) beats a months-long deletion before anything ships.

**Consequences:**
- v1 ships with AI/cloud code present-but-inert (larger binary; no AI/cloud UX).
- The `voice_input` crate is the one piece already deleted (it was a clean leaf).
- Future deletion, if pursued, is one dedicated AI+cloud branch effort.
- Anything removed remains restorable via [`REMOVED.md`](REMOVED.md) + `git revert`.

**Detail:** [`removal/ai-removal-feasibility.md`](removal/ai-removal-feasibility.md).

**Supersedes:** the earlier implicit assumption (Wave 0 / `05-removal-map.md`) that
de-Warp meant deleting the AI/cloud crates up front. Those removal specs remain
valid as the blueprint *if/when* full deletion is undertaken.

## ADR-006 — Branch model: `main` + `upstream` remote + `fork-base` tag
**Decision (2026-06-05):**
- **`main`** is Tarp's development branch and default (renamed from the working
  `dewarp` branch — its 15 commits *are* the mainline).
- **Upstream is a remote, not a maintained branch.** `upstream` →
  `https://github.com/warpdotdev/warp.git` (upstream's default branch is `master`).
  Sync by cherry-picking / path-scoped pulls from the **`upstream/master`
  remote-tracking ref** after `git fetch upstream` — no hand-maintained local mirror.
- **`fork-base`** tag marks the exact fork point (`2bb3a04b`) so the baseline is
  permanent and diffable.
- The old local `master` (pristine Warp snapshot) was deleted as redundant: it's an
  ancestor of `main`, preserved by the `fork-base` tag, and re-obtainable as
  `upstream/master`.

**Why:** standard `main` convention for our own dev; a remote-tracking ref is
cleaner and less error-prone than a local mirror branch you can accidentally commit
to; the tag guarantees the baseline never gets lost. Implements the sync strategy in
ADR-003 / `08-upstream-sync.md`.

**Remote rollout status (2026-06-05): ✅ complete.**
- `main` pushed to `origin` and set as the **default branch**; `origin/master`
  deleted; `fork-base` tag pushed; `upstream` remote set.
- The default-branch change is admin-only and the active `gh` account (`sri-vapi`)
  lacks admin on `ramsrib/tarp`; performed it by temporarily
  `gh auth switch --user ramsrib` (owner, has it in keyring) → `gh repo edit
  --default-branch main` + `git push origin --delete master` → switched back to
  `sri-vapi`. No lasting change to gh's active account.

## ADR-007 — Privacy-first: no telemetry, no trackers, no ToS/privacy policy
**Decision (2026-06-05):** Tarp is a plain, local terminal with **zero** telemetry,
analytics, crash-reporting, or tracking, and **no** Terms of Service or Privacy
Policy (it offers no cloud service, stores nothing remotely, tracks nothing).
- Telemetry network egress (`app/src/server/telemetry/mod.rs::send_batch_messages_to_rudder`,
  the single analytics POST chokepoint) is an **unconditional no-op** — nothing is
  transmitted regardless of channel config/settings (defense-in-depth on top of the
  OSS channel's `telemetry_config: None` and no analytics/sentry features compiled in).
- No crash-reporting/Sentry (features off); no firebase call on launch (anon-user
  creation only lives in the bypassed login flow).
- Removed the "Terms of Service" link; Privacy/Shared-blocks settings tabs and the
  Privacy-Policy menu item already removed.
**Why:** matches the project's reason to exist; eliminates the takedown/privacy risk
surface; "just a terminal."
**Detail:** `PROGRESS.md` (privacy entry).

## ADR-008 — Branding scope: convert user-exposed surfaces only; defer `WARP_*` env vars
**Decision (2026-06-05):** Rename "Warp"→"Tarp" only where it is **exposed to the
user**; leave internal code as-is.
- **Convert:** UI labels/strings, menus, notifications, settings, window title,
  About, app/bundle identity (`dev.tarp.Tarp`, binary `tarp`, `Tarp.app`),
  `TERM_PROGRAM=TarpTerminal`, XTVERSION `Tarp(version)`, log file `tarp.log`, config
  dir `~/.tarp`, `.desktop`, URLs that were Warp's → the Tarp repo.
- **Keep (not user-visible):** Rust identifiers/types/`const`s, `feature = "..."`
  flag names, crate names (`warp_core`, `warpui`, …), module/file names,
  `warpdotdev`/`docs.warp.dev` upstream URLs, telemetry payload strings (backend,
  and now inert), `warp-oss` artifact names in `autoupdate/*` (must match release
  artifacts; autoupdate disabled anyway), required Denver copyright.
- **Deferred:** the `WARP_*` shell-integration env vars **are** exposed (exported into
  the user's shell, visible via `env`), so they qualify — but renaming to `TARP_*` is
  a large, tightly-coupled change (~38 names, ~87 Rust read-sites + shell scripts +
  OSC markers) that risks breaking shell integration, with purely cosmetic benefit.
  **Deferred to a dedicated, well-tested pass** — see [`BACKLOG.md`](BACKLOG.md).
**Why:** maximizes branding cleanliness for what users see while avoiding risky,
zero-functional-value churn in internal plumbing (also keeps upstream-sync easier).

## ADR-009 — Distribution: tag-driven release, unsigned v1, signing behind secrets
**Decision (2026-06-05):** Ship downloadable builds via a **tag-driven GitHub
Actions release** (`v*` → build → publish Release), reusing the existing `oss`
bundle path; **v1 is unsigned**.
- **Trigger/scope:** push a `vX.Y.Z` tag (or manual dispatch). v1 builds **macOS
  arm64 only** (`./script/bundle --channel oss --nouniversal`) — the verified
  platform; Intel/universal + Linux/Windows are later matrix legs.
- **Licenses:** `THIRD_PARTY_LICENSES.txt` generated via `cargo-about` during the
  bundle (release deps), bundled in the app and attached to the Release.
- **Unsigned v1:** no Apple Developer Program cost; Gatekeeper warns on first launch
  — install docs cover right-click→Open / `xattr -dr com.apple.quarantine`. The
  workflow **auto-activates signing + notarization** when the `APPLE_*` repo secrets
  are present (no workflow edits); `APPLE_TEAM_ID` is now env-overridable in
  `script/macos/bundle` so a fork signs with its own Developer ID.
- **DMG hygiene:** the OSS DMG is built from a clean staging dir (only `Tarp.app`),
  volume name follows the app, and the upstream Warp install-background artwork is
  dropped (plain DMG).
- **No auto-update** (`autoupdate_config: None`); updates are manual via Releases.
**Why:** gets a real, downloadable artifact out cheaply and reproducibly, keeps all
release logic in the Tarp-owned `script/`/`.github/` surface (zero tracked-crate
edits, per the CI plan), and makes the signed path a pure secrets-config step later.
See [`../RELEASING.md`](../RELEASING.md) and [`removal/ci-plan.md`](removal/ci-plan.md).

**Update (2026-06-07):** signing is now **active** — `v0.1.0` ships **signed
(Developer ID) + notarized** (App Store Connect API key); the 7 `APPLE_*` secrets
are configured and the workflow auto-uses them (falls back to a valid ad-hoc
signature if absent — which shows the openable "Apple could not verify…" prompt,
not "damaged"). A **preflight gate** (fmt + compile check) now also gates the build.
Windows Authenticode remains TODO.

## ADR-010 — macOS-native UX defaults + searchable session palette
**Decision (2026-06-16):**
- **Defaults:** monospace font `SF Mono` (was Hack), size `16` (was 14), cursor
  `Block` (was Bar), cursor blink `off` (was on). (`app/src/settings/font.rs`,
  `app/src/settings/editor.rs`.)
- **Session palette:** search and display the user-set **tab title** and **tab
  color**. The title is prepended to the searchable string so it dominates fuzzy
  scoring; each row shows a color dot + `[title]` with matched chars bolded.
  (`app/src/session_management.rs` + its construction chain +
  `app/src/search/command_palette/navigation/{search,render}.rs`.)

**Why:**
- The defaults match common macOS terminal prefs (and the maintainer's Ghostty
  config); we ship **macOS-only** (ADR-009), so `SF Mono` is a reasonable default.
  It isn't bundled and isn't guaranteed installed, but font loading falls back to
  the embedded **Hack** when it's missing (`app/src/appearance.rs`) — worst case is
  "SF Mono if present, else Hack", never a crash or blank glyph. `copy_on_select`
  and confirm-on-quit already matched, so they were left alone.
- The session palette was effectively useless for the maintainer's workflow: every
  row rendered the same prompt + command, and the custom title — the one thing that
  distinguishes sessions — was neither searched nor shown. Prompt matches are noise
  when every session shares the prompt, hence **title-dominant** scoring.

**Consequences:** defaults apply to new installs and to existing users who never
overrode them (persisted `~/.tarp` values still win). Note the font is a **global**
appearance setting, theme-independent — there is no per-theme font.

## ADR-011 — Re-enable the CLI-agent rich-input composer (Ctrl-G); footer/chips stay off
**Decision (2026-06-16):** Allow **Ctrl-G** to open a multi-line **rich-input
composer** for a detected CLI coding agent (Claude Code, codex, gemini, …) while
keeping the standalone agent footer and toolbar chips **off** by default. Three
coordinated changes:
1. Add `cli_agent_rich_input` to the **`default`** cargo-feature set so
   `FeatureFlag::CLIAgentRichInput` is enabled — the open handler
   (`app/src/terminal/view/use_agent_footer/mod.rs`) bails otherwise. The
   rich-input *code* is always compiled; only the flag registration was gated.
2. Re-gate the Ctrl-G binding (`app/src/terminal/view/init.rs`) on an **active
   agent session** (`CLI_AGENT_SESSION_ACTIVE_KEY`, set whenever an agent command
   is detected) instead of the footer/chip context flags. So Ctrl-G opens the
   composer only when an agent runs in a long-running/alt-screen state, and still
   passes through as **BEL** for non-agent TUIs (vim/htop/less) and at a bare prompt.
3. Render the composer **chrome-free** (`app/src/terminal/input/cli_agent.rs`):
   drop the bottom toolbar (brand icon, file explorer, settings, attach, context
   chips). Close with Ctrl-G or Escape.

**Why:** the maintainer drives Claude Code from inside Tarp; a clean composer for
multi-line prompts is genuinely useful, but the full agent footer/toolbar is exactly
the "AI surface" Tarp strips. Gating on the agent-session context (not the footer
settings) yields the composer without the chrome, and without swallowing Ctrl-G
everywhere. Agent **detection** is upstream and not flag-gated — `"claude"` and the
other agent commands are in the built-in list (`app/src/terminal/cli_agent.rs`).

**Relationship to ADR-005 / ADR-007:** a **narrow, deliberate** re-opening of one
slice of the agent surface those ADRs disabled — scoped to a single keybinding +
composer, with the footer/chips and all other agent UI still off. Not a reversal of
the no-AI/no-cloud stance.

**Consequences:** enabling the flag also surfaces a couple of rich-input config rows
in the AI settings page (`app/src/settings_view/ai_page.rs`) — harmless. The A8
feature-flags spec lists `cli_agent_rich_input` under "REMOVE — AI/agents"; that
categorization is now superseded for the `default` set (noted in
[`removal/feature-flags.md`](removal/feature-flags.md)).
