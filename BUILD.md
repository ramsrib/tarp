# Building Tarp (from the Warp fork) on macOS

This documents a **verified** end-to-end build → bundle → launch of the OSS
channel on macOS (Apple Silicon), performed with **no source changes** on
2026-06-05. For the deeper audit see [`docs/`](docs/README.md); for the toolchain
deep-dive see [`docs/01-build-and-toolchain.md`](docs/01-build-and-toolchain.md).

> ✅ **Verified:** compiles, bundles into a codesigned `WarpOss.app`, and launches
> a working terminal window (Metal GPU renderer, runs logged-out). No
> Warp-private access was needed at any step.

## Prerequisites

| Requirement | Notes |
|---|---|
| **macOS** + Apple Silicon | Verified on Darwin 25.x / Apple Silicon. |
| **Full Xcode** (not just Command Line Tools) | Verified with Xcode 26.5. Required because `warpui` compiles Metal shaders. |
| **Metal Toolchain** (Xcode component, ~688 MB) | **Required** and easy to miss — `warpui`'s build script invokes `xcrun metal`. Install: `xcodebuild -downloadComponent MetalToolchain`. If that fails to load a plugin, run `xcodebuild -runFirstLaunch` first (worked without sudo here). |
| **Rust 1.92.0** | Pinned by `rust-toolchain.toml`; rustup auto-selects it. |
| **Homebrew** | Used by `script/macos/install_build_deps`. |
| **cargo-bundle** (pinned fork) | Needed only for the `.app` bundle step. Install with the exact rev below. |

### One-time setup (what bootstrap automates)

```sh
# 1. Xcode + Metal toolchain
xcodebuild -runFirstLaunch
xcodebuild -downloadComponent MetalToolchain
xcrun -sdk macosx metal --version    # sanity check

# 2. Pinned cargo-bundle (required to build the .app)
cargo install cargo-bundle \
  --git=https://github.com/burtonageo/cargo-bundle \
  --rev ae4c76e92c08774bf54ff077b1c52e3d1cd6c16d
```

`./script/bootstrap` does the above (plus brew/rust install) but uses `sudo` for
`xcode-select`; the steps above are the minimum needed if your Xcode is already
selected.

## Build only (binary, no app bundle)

```sh
cargo build --bin warp-oss --features gui
# → target/debug/warp-oss   (Mach-O arm64; ~721 MB debug, unstripped)
```

- All ~810 crates compile, including the warpdotdev git dependencies. They fetch
  over **anonymous HTTPS** — no auth/SSH needed.
- Cold compile is ~3–4 min on Apple Silicon (the `app` crate links last and is the
  heaviest unit).

## Build + bundle + run (the real app)

```sh
# Bundle a codesigned WarpOss.app and launch it:
WARP_SKIP_COMMON_SKILLS_INSTALL=1 ./script/run

# Bundle without launching:
WARP_SKIP_COMMON_SKILLS_INSTALL=1 ./script/run --dont-open
# → target/debug/bundle/osx/WarpOss.app
```

Notes:
- **Channel selection is automatic.** With no `warp-channel-config` on `PATH`,
  `./script/run` builds the **OSS** channel (`warp-oss` binary, channel `oss`).
  This is the natural build for contributors and for Tarp. (You'll see
  `Cannot access ...warp-channel-config.git ... Skipping install.` — that's
  expected and harmless.)
- `WARP_SKIP_COMMON_SKILLS_INSTALL=1` avoids a network skills-install step that
  isn't needed to build/run.
- The bundle step (`cargo bundle`) recompiles the top crates the first time, then
  runs `update_plist`, `prepare_bundled_resources`, icon compile, and `codesign`
  (ad-hoc if no Apple Development cert is found).

### What a successful launch looks like

The app spawns a main process + a `terminal-server` child, initializes SQLite/
settings, computes fonts, brings up the **Metal GPU renderer**, and opens a window:

```
[INFO] [warp] Starting warp with channel state ... channel: Oss ...
[INFO] [warpui::platform::mac::window] Opening window with id 0
[INFO] [warpui::platform::mac::window] Using discrete GPU for rendering new window.
```

It runs **logged-out** without an account
(`Unable to read user from secure storage: ... NotFound` is expected).

Log file: `~/Library/Logs/warp-oss.log`.

## Useful commands

| Command | Purpose |
|---|---|
| `cargo build --bin tarp --features gui` | Build the OSS binary. |
| `./script/run` | Build + bundle + launch (OSS). |
| `./script/run --dont-open` | Build + bundle, don't launch. |
| `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2` | Test suite. |
| `./script/presubmit` | fmt + clippy + tests. |
| `./script/format` | Format. |

## Producing a release build (DMG)

The commands above produce a fast **debug** app for local testing. A distributable,
release-optimized DMG is produced by the bundle script's `oss` channel:

```sh
brew install create-dmg                              # one-time
./script/bundle --channel oss --nouniversal --nosign # unsigned arm64 DMG
# → target/release-lto/bundle/osx/Tarp.dmg
```

In practice you don't run this by hand — pushing a `vX.Y.Z` tag runs it on CI and
publishes a GitHub Release. See [`RELEASING.md`](RELEASING.md) for the tag-driven
flow, signing/notarization, and the unsigned-app install workaround.

## Known gotchas

1. **"cannot execute tool 'metal' … missing Metal Toolchain"** during `warpui`
   build → install the Metal Toolchain (see prerequisites).
2. **`error: no such command: bundle`** → install the pinned `cargo-bundle`.
3. **`xcodebuild` plugin load failure** (DVTDownloads/IDESimulatorFoundation
   symbol mismatch) → run `xcodebuild -runFirstLaunch`, then retry the download.
4. **`Cannot access …warp-channel-config.git`** → expected; you get the OSS build.
5. **Server URLs in the log** (`app.warp.dev`, firebase, `oz.warp.dev`) are the
   cloud/account wiring that the Tarp de-Warp work removes — see
   [`docs/05-removal-map.md`](docs/05-removal-map.md).
