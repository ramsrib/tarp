# Tarp — Backlog / Deferred Work

Known, deliberately-deferred items. Each notes *why* it's deferred and what doing it
entails. Newest concerns first. See [`DECISIONS.md`](DECISIONS.md) for rationale and
[`PROGRESS.md`](PROGRESS.md) for what's done.

## Branding / identity

- **Revisit the default themes.** Only the default (Adeberry) theme has been
  Tarp-tuned so far — its cursor is set to the brand olive-green (`#7E9B3F`) instead
  of falling back to its steel-blue accent (`app/src/themes/default_themes.rs`). The
  other bundled themes still carry upstream Warp palettes; do a pass to align/curate
  them (cursor colors, accents, naming) to Tarp's look.
- **Rename `WARP_*` shell-integration env vars → `TARP_*`.** They ARE user-exposed
  (exported into the shell; visible via `env`), so per ADR-008 they qualify. Deferred
  because it's a large, tightly-coupled change — ~38 distinct names across the shell
  bootstrap scripts **and** ~87 Rust read-sites **and** the OSC/escape-sequence
  markers — all must change in lockstep or shell integration breaks (blocks, prompt,
  session tracking, SSH). Benefit is purely cosmetic (no leak/transmit). **Do as an
  isolated worktree pass with a real shell-integration test** (open bash/zsh/fish,
  run commands, verify blocks/prompt/SSH) before merging.
- **Real Tarp logo/wordmark asset.** About page + app icon currently use the
  generated icon / "Tarp" text. A proper wordmark SVG would replace the bundled
  `bundled/svg/tarp-logo.png` and could restore a styled wordmark on About.
- **Retina/adaptive macOS icon.** Current `.icns` is built from a 512px source
  (cargo-bundle rejected a multi-size list); a 1024 + `.icon` adaptive bundle is a
  polish item.
- **`WARP.md` / `FAQ.md`** are still Warp-branded engineering/product docs — rewrite
  or retire (BUILD.md + CONTRIBUTING.md + docs/ cover the essentials now).

## Release engineering (M4)

- ✅ **DONE — Tag-driven release workflow + first release.**
  `.github/workflows/release.yml`: `v*` tag → build `oss` bundle (macOS arm64) →
  generate `THIRD_PARTY_LICENSES` via `cargo-about` → publish GitHub Release. Unsigned
  v1 (ADR-009). See `RELEASING.md`. **`v0.1.0` shipped + smoke-tested** (downloads,
  installs via the documented `xattr` step, launches; version reads 0.1.0).
- ✅ **DONE — Release preflight gate.** `release.yml` now has a fast Linux job
  (`cargo fmt --check` + `cargo check --features gui`) gating the macOS build, so a
  release can't be cut from an unformatted/non-compiling commit. CI Linux + preflight
  share a `linux` rust-cache key (`cache-all-crates` on all caches).
- **macOS compile-check in CI.** `ci.yml` only builds **Linux** today, so mac-only
  breakage (objc/Metal/`platform/mac`) surfaces only at release time. Add a macOS
  `./script/bundle --channel oss --nouniversal --check-only` job (cheap `cargo check`
  with release features) to PR CI. (Costs macOS minutes per push — weigh it.)
- **Ad-hoc codesign seal warning.** `codesign --verify` on the unsigned bundle prints
  "code has no resources but signature indicates they must be present" (benign; app
  still launches via the documented workaround). Goes away when real Developer ID
  signing is enabled — no action needed until then.
- **Code signing + notarization.** *Wired but inactive.* The release workflow
  auto-signs+notarizes once the `APPLE_*` repo secrets are configured (needs an Apple
  **Developer ID Application** cert; `APPLE_TEAM_ID` is now env-overridable in
  `script/macos/bundle`). Until then, downloads are unsigned (Gatekeeper workaround
  in docs). Windows Authenticode still TODO.
- **Expand the release matrix.** v1 is macOS **arm64 only** (`--nouniversal`). Add:
  Intel/universal2 macOS, Linux (`script/linux/bundle` → deb/AppImage), Windows
  (`script/windows/bundle.ps1` → Inno `.exe`) — each once build-verified for Tarp.
- **Auto-update:** none today (`autoupdate_config: None`; the updater pointed at
  Warp's server). Decide: manual updates via GitHub Releases, or build a
  GitHub-Releases-based self-updater.
- ✅ **DONE — slim CI green.** `.github/workflows/ci.yml` (rustfmt + Linux build) is
  green on `main` after the `cargo fmt` import-ordering fix. Still TODO: promote clippy
  to `-D warnings` after the inert-cfg cleanup below.

## Deeper source removal (M6 — optional)

- **Full AI/cloud source deletion.** Current approach is *disable* (features off,
  UI stripped), not *delete* — the `ai`/`billing`/`pricing`/`drive`/`voice` modules,
  the alternate `agent_input_footer` path, cloud crates, etc. are still compiled in.
  Full deletion is a ~600–700-file coordinated AI+cloud surgery with high upstream
  divergence — see ADR-005 and `docs/removal/ai-removal-feasibility.md`. Pursue only
  if binary-size / source-purity justifies the cost.
- **Inert-cfg cleanup.** ~333 `#[cfg(feature = "voice_input")]` dead sites + other
  dead menu/settings constructions left to minimize diff; sweep when the owning
  surfaces are deleted.
- **`paths_tests.rs` etc.** assertions still reference `.warp-oss` after the config
  dir rename — update (not run by `cargo build`).

## Open product decisions (low priority)

- "Tarpify" verb — confirm vs. a neutral phrase.
- Telemetry payload description strings — currently left (backend, inert). Rebrand
  only if desired.
