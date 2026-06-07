# Tarp — Agent Handoff / Onboarding

Read this first. It orients a new agent (or human) on **what Tarp is, where things
are, how to work on it, and what's left**. Tarp is a fork of the Warp terminal,
stripped to a plain, modern, privacy-first terminal: **no AI, no cloud, no accounts,
no telemetry**.

- Repo: `https://github.com/ramsrib/tarp` · default branch **`main`** · platform macOS (Apple Silicon).
- Local path: `/Users/sriram/Projects/scratch/tarp`.

## Start here — the doc set (`docs/`)
| Doc | Why |
|---|---|
| [`docs/README.md`](docs/README.md) | Index of the audit/design docs (`01`–`09`). |
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | **ADR-001…008** — the consequential calls + rationale. Read before changing direction. |
| [`docs/PROGRESS.md`](docs/PROGRESS.md) | Chronological worklog (newest on top) — what's been done. |
| [`docs/BACKLOG.md`](docs/BACKLOG.md) | Deliberately-deferred work (what to do next). |
| [`docs/REMOVED.md`](docs/REMOVED.md) | Registry of every removal → restore via `git revert <commit>`. |
| [`docs/removal/`](docs/removal/README.md) | File-level removal specs (the de-Warp blueprint) + critic review. |
| [`BUILD.md`](BUILD.md) | How to build/run/bundle. |
| [`TARP-PLAN.md`](TARP-PLAN.md) | Milestones (M0–M6). |

## Current state (what's done)
- **Shipped `v0.1.0`** — first public release (macOS arm64), **signed (Developer ID)
  + notarized**: tag-driven `release.yml` (preflight gate → build → publish) builds
  the OSS DMG + `THIRD_PARTY_LICENSES`. Gatekeeper accepts it (`spctl` → Notarized
  Developer ID); downloads open with just the standard "Open" dialog. Smoke-tested
  end-to-end. Slim `ci.yml` (rustfmt + Linux build) green.
- **Default look/UX:** **Rainfly** theme (dark + light, olive-green accent/cursor,
  no Warp bg image), themed text selection, shell **PS1** prompt by default, 14px
  font, no per-block timer, agent footers off. (Themes/inputs are defaults — see
  PROGRESS / DECISIONS.)
- **Builds, bundles, launches** as `Tarp.app` (OSS channel). Identity is fully Tarp:
  `dev.tarp.Tarp`, binary `tarp`, `Tarp.app`, `TERM_PROGRAM=TarpTerminal`,
  config `~/.tarp`, log `tarp.log`, custom icon.
- **De-Warp by DISABLE, not delete** (ADR-005): the minimal `default` feature set
  (187→49) turns AI/cloud/editor off; the visible UI surfaces (startup login modal,
  menus, settings tabs, header, sidebar, input-bar AI affordances, command palette,
  context menus) were removed/gated. The `ai`/`cloud_*`/`drive`/etc. crates are still
  *compiled in* (source deletion is the optional M6).
- **Privacy-first** (ADR-007): telemetry network egress is a hard no-op; no trackers;
  no ToS/privacy policy; nothing phones home on launch.
- **~466 user-visible "Warp"→"Tarp" strings** converted across 151 files (ADR-008).
- **Branding**: app icon + README logo + social banner; regenerate via
  `script/gen_brand_assets.sh`.

## How to build / run / test
```sh
cargo build --bin tarp --features gui        # build the OSS binary
WARP_SKIP_COMMON_SKILLS_INSTALL=1 ./script/run --dont-open   # build + bundle Tarp.app (no launch)
./script/presubmit                            # fmt + clippy + tests
```
**Gotchas (all real, hit during this work):**
- Needs **full Xcode + the Metal Toolchain** (`xcodebuild -downloadComponent MetalToolchain`; run `-runFirstLaunch` first if it errors) and the **pinned `cargo-bundle`** — see BUILD.md.
- The OSS channel is auto-selected (no `warp-channel-config`); `Cannot access …warp-channel-config.git` is expected.
- Dev `./script/run` uses `NO_LICENSES=1` (skips license bundling) — license files ship only in the **release** bundle.
- This is a **debug** build: `debug_assert!`s are live (one bit us in `app_menus.rs`), and `cfg!(debug_assertions)` enables debug overlays — both off in release.
- Verifying a GUI change: install to `/Applications` (`cp -R target/debug/bundle/osx/Tarp.app /Applications/ && xattr -dr com.apple.quarantine /Applications/Tarp.app && open …`). macOS **caches icons** — `killall Dock` to refresh.

## Branch model & upstream sync (ADR-006, `docs/08`)
- `main` = Tarp dev (default). `upstream` remote → `warpdotdev/warp` (default branch
  `master`). `fork-base` tag marks the fork point.
- Pull upstream fixes by **cherry-pick / path-scoped** of terminal-core paths only —
  **never a full merge** (upstream is ~14 commits/day, mostly AI/cloud). Keep
  terminal-core crates close to upstream.

## Conventions for working here (IMPORTANT)
- **Disable, don't delete** (default-feature gating + UI removal) unless explicitly
  doing M6. Minimizes upstream divergence.
- **Edit `app/src/**` (Tarp-owned).** Avoid editing tracked terminal-core crates
  (`warp_core`, `warpui*`, `warp_terminal`, `editor`, `command`, `warp_completer`,
  `vim`, `syntax_tree`); every edit there is a future merge conflict. The few
  necessary tracked edits are logged (warpui menu strings, `warp_core` paths/channel
  identity) — see REMOVED.md / DECISIONS.md.
- **Branding scope** (ADR-008): change only what the user sees. Leave Rust
  identifiers, `feature="..."` names, crate names, `warpdotdev` URLs, and the
  `WARP_*` shell-integration env vars (deferred — BACKLOG). Keep the required
  "Denver Technologies, Inc." copyright (AGPL/MIT).
- **Commits:** small savepoints, build-verified. **No AI/Claude attribution, no
  Co-Authored-By** (user's global rule). Branch off `main` for substantial work.
- **Big sweeps:** the pattern that worked = a worktree-isolated agent that converts/
  removes to a green build + reports a diff, then merge (ff) + build + launch-verify.
- **Brand assets:** never hand-edit the generated PNGs — change
  `app/channels/oss/icon/AppIcon-source.png` and run `script/gen_brand_assets.sh`.

## What's next (see `docs/BACKLOG.md`)
1. **Release follow-ups (M4):** done — preflight gate, signed+notarized macOS
   build. Remaining: a **macOS compile-check in CI** (CI builds Linux only today),
   **expand the matrix** (Intel/universal2, Linux, Windows + Windows Authenticode),
   and an **auto-update** decision (none today).
2. **`WARP_*` → `TARP_*`** env-var rename (exposed but deferred — large/risky; do as
   an isolated pass with a shell-integration test).
3. Logo/wordmark polish; retire `WARP.md`/`FAQ.md`.
4. **M6 (optional):** full AI+cloud *source* deletion (~600–700 files, high
   divergence — only if binary-size/source-purity justifies it).

## Owner preferences (observed)
Privacy-first, minimalist ("just a terminal"); fewer/stable releases (burst early →
~monthly); aggressive de-branding of anything user-visible; keep internal code intact;
wants decisions/removals/findings captured in docs as work proceeds.
