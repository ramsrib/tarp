# 01 — Build & Toolchain

How to build the fork from source on macOS, and the result of an actual build run
on **2026-06-05** with **zero code changes**.

## Headline result

> ✅ **The fork builds from source with no code changes and no Warp-private
> access.** All ~810 dependency + workspace crates — including the 5
> formerly-"private" warpdotdev crates — compile. The only setup hurdle was a
> standard macOS prerequisite (the Metal Toolchain), not anything Warp-specific.

(Final binary/bundle status recorded in [§Build run](#build-run) below.)

## Toolchain

| Tool | Required | On this machine |
|---|---|---|
| Rust | `1.92.0` (pinned in `rust-toolchain.toml`, components `rustfmt`+`clippy`) | rustup auto-selected 1.92.0 ✓ |
| Xcode | full Xcode (not just CLT) | Xcode 26.5 (17F42) at `/Applications/Xcode.app` ✓ |
| **Metal Toolchain** | Xcode downloadable component — **required** to compile GPU shaders | Was missing; installed during this audit ✓ |
| Homebrew | for `install_build_deps` | present ✓ |
| `cargo-bundle` | pinned fork (`burtonageo/cargo-bundle@ae4c76e`) for `--profile` support | needed for `.app` bundling |

## Setup path (what `./script/bootstrap` does on macOS)

`script/macos/bootstrap` →
1. Ensures full Xcode is selected (`sudo xcode-select --switch`), runs
   `xcodebuild -runFirstLaunch`.
2. Installs brew + rust if missing.
3. `install_cargo_test_deps`, `install_cargo_release_deps`, the pinned `cargo-bundle`.
4. `brew update`.

`script/macos/install_build_deps` →
- `xcodebuild -downloadComponent MetalToolchain` ← **the critical, easily-missed step.**

> **Gotcha hit during this audit:** `xcodebuild -downloadComponent MetalToolchain`
> first failed because Xcode couldn't load a plugin (`DVTDownloads`/
> `IDESimulatorFoundation` symbol mismatch between Xcode.app and the OS
> frameworks). The fix was `xcodebuild -runFirstLaunch` (succeeded **without
> sudo** here), after which the Metal Toolchain (687.9 MB) downloaded cleanly and
> `xcrun -sdk macosx metal --version` worked. This matches what `bootstrap` does —
> we just hit it manually because we skipped bootstrap's sudo steps.

## Build & run commands

| Command | Effect |
|---|---|
| `cargo build --bin warp-oss --features gui` | Builds the **OSS channel** binary (what we used). |
| `./script/run` | Builds + bundles + launches. With no `warp-channel-config` on PATH it auto-selects the **OSS** channel (`warp-oss`, channel `oss`) and `gui` feature. |
| `cargo build` | Builds `default-members` only (lean subset). |
| `cargo nextest run --workspace --exclude command-signatures-v2` | Test suite. |
| `./script/presubmit` | fmt + clippy + tests. |

Channel selection logic (`script/run`): if `warp-channel-config` is present →
`warp` binary / `local` channel; otherwise → **`warp-oss` binary / `oss` channel**.
This is why an outside contributor naturally builds the OSS variant — convenient
for Tarp.

### Server features (not needed for a plain terminal)
`with_local_server`, `with_local_session_sharing_server`,
`with_sandbox_telemetry` are no longer cargo features — `script/run` maps them to
env vars (`WITH_LOCAL_SERVER=1`, …). A pure-terminal Tarp leaves these unset.

## Build run

- **Command:** `cargo build --bin warp-oss --features gui` (debug profile).
- **Environment:** macOS (Apple Silicon), Xcode 26.5, Rust 1.92.0.
- **Dependency fetch:** all git deps resolved over anonymous HTTPS, including the
  5 formerly-"private" warpdotdev repos and several transitive warpdotdev forks
  (`difflib`, `rust-email_address`, `uneval`). **No auth required.**
- **First attempt:** compiled **809 crates** then failed only at
  `warpui/build.rs:94` — `metal` shader compile — because the Metal Toolchain was
  absent. **1 warning, 0 code/dependency errors.**
- **After installing the Metal Toolchain:** build resumed and compiled the
  workspace crates (`ai`, `cloud_objects`, `warp_server_auth`, …, `app`).

### ✅ Final result — SUCCESS

| | |
|---|---|
| Cargo exit code | **0** |
| Compile errors | **0** (1 warning) |
| Result line | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 16s` |
| Output binary | `target/debug/warp-oss` |
| Binary size | **721 MB** (debug, unstripped) |
| Binary type | `Mach-O 64-bit executable arm64` |

Cold build wall-clock was roughly **~3.5 min of compilation total** (≈82s for the
first ~809 crates, then ≈136s to finish the workspace crates + link `app` after
the Metal Toolchain was installed) on Apple Silicon. The `app` crate links last
and is the heaviest single unit.

> The OSS `warp-oss` binary builds end-to-end from a clean fork with **no code
> changes**.

### ✅ Bundle + launch — also verified

After installing the pinned `cargo-bundle`,
`WARP_SKIP_COMMON_SKILLS_INSTALL=1 ./script/run --dont-open` produced a codesigned
**`target/debug/bundle/osx/WarpOss.app`** (exit 0), and `open`-ing it launched a
working terminal:

- main process + a `terminal-server` child process spawned;
- SQLite/settings initialized, fonts computed;
- **Metal GPU renderer** came up and a window opened
  (`Opening window with id 0` / `Using discrete GPU for rendering new window`);
- runs **logged-out** (`Unable to read user from secure storage: NotFound`).

The OSS build still references `app.warp.dev` / firebase / `oz.warp.dev` (cloud +
account wiring — removal targets), while `telemetry_config`, `autoupdate_config`,
and `crash_reporting_config` are already `None` on the OSS channel. Full
build/bundle/run instructions are in [`../BUILD.md`](../BUILD.md).

## Takeaways for the plan

1. **No source or dependency blockers exist.** The "private deps will block the
   build" worry is dead — they're public and compile.
2. **The only real prerequisite friction is the Metal Toolchain** (a 688 MB Xcode
   component) — already handled by `script/macos/install_build_deps`. Document it
   prominently in Tarp's `BUILD.md` because it's easy to miss and the error message
   is opaque.
3. **Full Xcode is required** (not just Command Line Tools) due to the Metal
   shader compilation in `warpui`.
4. **The OSS channel is the natural build target** for contributors and for Tarp.
