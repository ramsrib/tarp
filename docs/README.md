# Tarp — Fork Audit & Build Documentation

This `docs/` set captures a full audit of the Warp fork (`ramsrib/tarp`) as it
exists today — an **unmodified copy of upstream Warp** at commit `2bb3a04b` — and
what it takes to turn it into Tarp: a plain, publishable, open-source terminal
with AI, cloud, accounts, and other non-terminal concerns removed.

All findings here are from inspecting the repo on **2026-06-05**. No source code
has been changed; this is documentation only.

## How to read this

| Doc | What's in it |
|---|---|
| [`01-build-and-toolchain.md`](01-build-and-toolchain.md) | How to build the fork from source, toolchain, system deps, and the result of a real build run. |
| [`02-architecture-and-crates.md`](02-architecture-and-crates.md) | Workspace layout, all 69 crates + `app/`, what each does, sizes. |
| [`03-dependencies.md`](03-dependencies.md) | External dependencies, the 22 git deps, and the 5 formerly-"private" warpdotdev crates (now public). |
| [`04-licensing.md`](04-licensing.md) | License model (AGPL + MIT), third-party attribution tooling, audit findings & required fixes. |
| [`05-removal-map.md`](05-removal-map.md) | The de-Warp surface: AI, cloud, accounts, telemetry — crate-level and `app/`-level reach, plus the 292 cargo feature flags. |
| [`06-branding-and-rename.md`](06-branding-and-rename.md) | Warp branding inventory, bundle IDs, channels, shell integration, what to rename for Tarp. |
| [`07-ci-and-release.md`](07-ci-and-release.md) | CI workflows, packaging scripts, release machinery — keep/cut/replace. |
| [`08-upstream-sync.md`](08-upstream-sync.md) | How to selectively pull fixes/features from upstream Warp without inheriting the AI/cloud churn. |
| [`09-parallel-execution-plan.md`](09-parallel-execution-plan.md) | Agent-driven execution: what parallelizes, the analysis fan-out, and the sequenced de-Warp surgery. |
| [`DECISIONS.md`](DECISIONS.md) | Decision log (ADR-style) — the consequential calls and their rationale. |
| [`REMOVED.md`](REMOVED.md) | Removed-features registry with restore pointers. |
| [`PROGRESS.md`](PROGRESS.md) | Chronological work log. |

The actionable, sequenced plan lives in [`../TARP-PLAN.md`](../TARP-PLAN.md).
This doc set is the **evidence** behind that plan.

## Headline numbers

- **1,393,807** lines of Rust across **3,411** files (`crates/` + `app/`).
- **69** workspace crates + the `app/` binary crate (2,093 files alone).
- **1,507** total resolved packages in `Cargo.lock`.
- **22** external git dependencies (16 warpdotdev/servo forks; more pulled transitively).
- **292** cargo feature flags in `app/`, with a ~190-entry `default` set dominated by AI/cloud/agent features.
- **Dual license**: `warpui` + `warpui_core` are MIT; everything else AGPL-3.0-only.

## The single most important finding

The de-Warp effort is **feature-flag and `app/`-layer surgery, not crate deletion.**
The removable functionality (AI, cloud, sharing, accounts, telemetry) is woven
through hundreds of files in `app/` (629 reference AI, 356 auth, 315 telemetry,
253 cloud objects) and gated behind ~190 default-on cargo features — not isolated
in the handful of obviously-named crates. See [`05-removal-map.md`](05-removal-map.md).
