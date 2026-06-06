# Tarp — Backlog / Deferred Work

Known, deliberately-deferred items. Each notes *why* it's deferred and what doing it
entails. Newest concerns first. See [`DECISIONS.md`](DECISIONS.md) for rationale and
[`PROGRESS.md`](PROGRESS.md) for what's done.

## Branding / identity

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

- **Code signing + notarization.** Builds are ad-hoc signed only (Gatekeeper rejects
  downloaded copies). Needs an Apple **Developer ID Application** cert + notarization
  (macOS) and an Authenticode cert (Windows), or ship unsigned with install docs.
- **Tag-driven release workflow** + packaging (reuse `script/{macos,linux,windows}/bundle*`)
  + generated `THIRD_PARTY_LICENSES` bundled in the artifact (the dev build uses
  `NO_LICENSES=1`). See `docs/removal/ci-plan.md`.
- **Auto-update:** none today (`autoupdate_config: None`; the updater pointed at
  Warp's server). Decide: manual updates via GitHub Releases, or build a
  GitHub-Releases-based self-updater.
- **Verify the slim CI** (`.github/workflows/ci.yml`) actually goes green on first
  push; promote clippy to `-D warnings` after the inert-cfg cleanup below.

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
