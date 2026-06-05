# A6 — Branding Map (Warp → Tarp)

Exhaustive, file-level branding inventory with proposed Tarp replacements. This
is the **execution-grade** companion to the higher-level
[`../06-branding-and-rename.md`](../06-branding-and-rename.md): that doc gives the
rationale and checklist; this one gives `file:line` targets, the exact strings,
and a sequenced worklist an implementer can run mechanically.

Findings from inspecting the repo on 2026-06-05. Read-only; nothing changed.

## Scope

In scope: bundle identifiers, channel definitions, app/binary names, the
DockTilePlugin, `.desktop` files + icons, `images/`/`resources/` brand assets,
the Warp-glyph font patch, `about.hbs`/`about.toml`, user-facing "Warp" strings
(settings/about/menus/notifications/window title), `warp.dev`/`docs.warp.dev`
URLs, and shell-integration markers/env vars in
`app/assets/bundled/bootstrap/*`.

Out of scope (other A-tasks): removing AI/cloud/account/telemetry code; license
changes (A4); CI/release workflow surgery (A7). Branding strings that live
**inside** removed surfaces (e.g. `app/src/ai/**`, onboarding banners) disappear
for free when those surfaces are deleted — they are flagged below but should not
be edited by hand.

## Proposed Tarp naming (single source of truth for the worklist)

| Concept | Warp value | Proposed Tarp value |
|---|---|---|
| Reverse-DNS qualifier.organization | `dev.warp` | `dev.tarp` (or `sh.tarp`) |
| App display name | `Warp` / `WarpOss` | `Tarp` |
| macOS bundle id | `dev.warp.WarpOss` | `dev.tarp.Tarp` |
| Unix binary name | `warp-oss` / `warp` | `tarp` |
| Linux launcher / pkg name | `warp-terminal` | `tarp` |
| URL scheme | `warposs` / `warp` | `tarp` |
| Docs URL | `docs.warp.dev` | (Tarp docs, or drop the link) |
| Copyright | `© 2025, Denver Technologies, Inc` | keep Denver notice (AGPL) **+** add Tarp |

Shell-integration `WARP_*` env vars: see the dedicated section — the recommendation
is to **NOT rename them** to avoid churn in tracked bootstrap scripts.

---

## Recommendation up front: collapse to ONE channel

Warp ships 5 channels (`stable`, `preview`, `dev`, `local`, `oss`) plus
`integration`. Each has its own bin target, bundle id, name, icon set, `.desktop`
file, and URL scheme. Tarp is a single open-source terminal and only needs the
`Oss` channel.

**Collapse plan:** keep `Channel::Oss` as the lone shipping channel (rename its
strings to Tarp). Keep `Channel::Integration` only if the integration test suite
is retained (it's internal, never user-facing). Delete `stable`/`preview`/`dev`/
`local` bin targets, their bundle metadata, their `app/channels/{stable,preview,dev,local}/`
dirs, and their `[package.metadata.bundle.bin.*]` blocks.

This deletes the Sentry `osx_frameworks` references (telemetry, A1) as a side
effect, and removes 4 of 5 `.desktop` files and 4 of 5 icon sets.

> **Tracked-crate caveat:** the `Channel` enum lives in `warp_core`
> (tracked-from-upstream). Removing variants forces edits to a tracked crate and
> every `match Channel { … }` arm. **Lower-risk alternative:** keep the enum
> intact, keep only the `Oss` bin target, and just rename the Oss strings. This
> keeps `warp_core` structurally upstream-shaped (see Risks).

---

## Group 1 — Bundle identity in `app/Cargo.toml` (Tarp-owned)

Five `[package.metadata.bundle.bin.*]` blocks, `app/Cargo.toml`:

| Block | Lines | identifier | name |
|---|---|---|---|
| `warp-oss` | 998–1005 | `dev.warp.WarpOss` (`:1001`) | `WarpOss` (`:1002`) |
| `stable` | 1007–1016 | `dev.warp.Warp-Stable` (`:1010`) | `Warp` (`:1011`) |
| `preview` | 1018–1027 | `dev.warp.Warp-Preview` (`:1021`) | `WarpPreview` (`:1022`) |
| `dev` | 1029–1038 | `dev.warp.Warp-Dev` (`:1032`) | `WarpDev` (`:1033`) |
| `warp` (local) | 1040–1046 | `dev.warp.Warp-Local` (`:1043`) | `WarpLocal` (`:1044`) |

Also in each block:
- `copyright = "© 2025, Denver Technologies, Inc"` (`:1000, :1009, :1020, :1031, :1042`).
- `icon = ["channels/oss/icon/no-padding/512x512.png", "…/icon.ico"]` (`:1004`, oss only).
- `short_description = "The open-source, cloud-backed terminal…"` (`:1005`, et al.) — drop "cloud-backed".
- `osx_frameworks = [… Sentry.framework]` (`:1012-1014, :1023-1025, :1034-1036`) — Sentry; deleted with telemetry/channels.

`[[bin]]` targets, `app/Cargo.toml`:
- `name = "warp"` package (`:7`) and lib `name = "warp"` (`:13`) — the crate name.
  Renaming the crate is invasive (every `warp::` path, `use warp::…`); **recommend
  leaving the Rust crate name `warp`** and only renaming the *binary/product*.
- bin `warp-oss` → src/bin/oss.rs (`:21`).
- bin `warp` → src/bin/local.rs (`:26`) — delete with channel collapse.
- bin `stable`/`dev`/`preview` (`:36/:41/:46`) — delete with channel collapse.
- bin `integration` (`:31`), `generate_settings_schema` (`:52`) — keep (internal).

**Action:** rewrite the `warp-oss` block → `dev.tarp.Tarp` / `name = "Tarp"`;
delete the other four bundle blocks and their `[[bin]]` targets.

## Group 2 — The OSS entrypoint plist (Tarp-owned, highest priority)

`app/src/bin/oss.rs` — the canonical Tarp build entrypoint, contains an embedded
`Info.plist` (lines 34–67):
- `AppId::new("dev", "warp", "WarpOss")` (`:14`) → `("dev", "tarp", "Tarp")`.
- `logfile_name: "warp-oss.log"` (`:15`) → `"tarp.log"`.
- `CFBundleDisplayName` / `CFBundleName` = `WarpOss` (`:42, :50`) → `Tarp`.
- `CFBundleExecutable` = `warp-oss` (`:44`) → `tarp`.
- `CFBundleIdentifier` = `dev.warp.WarpOss` (`:46`) → `dev.tarp.Tarp`.
- `CFBundleURLSchemes` = `warposs` (`:62`) → `tarp`.
- `NSHumanReadableCopyright` = `© 2026, Denver Technologies, Inc` (`:64`) — keep + add Tarp.
- `server_config: WarpServerConfig::production()`, `oz_config: OzConfig::production()`
  (`:16-17`) — these point at `app.warp.dev`/`oz.warp.dev`; obsolete once cloud is
  removed (A2/A5), but leaving them is harmless until then.

Sibling entrypoints to **delete** with the channel collapse: `app/src/bin/local.rs`
(`dev.warp.Warp-Local`), `dev.rs`, `preview.rs`, `stable.rs`. Keep
`integration.rs` (also constructs an `AppId` with `dev`/`warp`, but internal).

## Group 3 — Channel identity in tracked `warp_core` (⚠ tracked crate)

These encode brand strings inside a tracked-from-upstream crate. **Every edit
here is a merge-conflict risk.** Minimize.

`crates/warp_core/src/channel/mod.rs`:
- `enum Channel { Stable, Preview, Dev, Local, Oss, Integration }` (`:10-26`).
- `cli_command_name()` (`:51-60`): returns `"oz"`, `"oz-dev"`, …, `"warp-oss"`
  (`:53-58`). The Oss arm `:58` `"warp-oss"` → `"tarp"`.
- `Display` (`:63-74`): Oss → `"warp-oss"` (`:71`) → `"tarp"`.

`crates/warp_core/src/channel/state.rs`:
- fallback `AppId::new("dev", "warp", "WarpOss")` (`:40`) → `("dev", "tarp", "Tarp")`.
- `url_scheme()` (`:384-394`): per-channel schemes `warp`/`warppreview`/`warpdev`/
  `warplocal`/`warposs` (`:386-392`). Oss arm `:392` `"warposs"` → `"tarp"`.

`crates/warp_core/src/app_id.rs` — generic `AppId` (qualifier/org/app); no literal
brand strings, no edits needed.

`crates/warp_core/src/channel/config.rs` — `WarpServerConfig::production()`
(`:57-65`) hardcodes `app.warp.dev`, `rtc.app.warp.dev`, `sessions.app.warp.dev`,
a `firebase_auth_api_key`; `OzConfig::production()` (`:80-85`) hardcodes
`oz.warp.dev`. These are **cloud config**, removed by A2/A5, not branding — but
they are `warp.dev` URLs living in a tracked crate. Flag for A2.

`crates/warp_core/src/paths.rs` — derives the on-disk data dir from the app name;
contains a special-case mapping (the OSS `application_name` → `"Warp-Oss"`
directory). Renaming the app name changes the user-data path
(`~/Library/Application Support/WarpOss` → `Tarp`). Update the mapping here and in
`paths_tests.rs` (asserts `WarpOss`/`WarpDev`/`Warp` dir names). ⚠ tracked.

> **Strategy:** do all of Group 3 in **one small commit** isolated from other
> changes, so a future upstream sync can re-apply or drop it cleanly. Prefer the
> single-channel approach (touch only the `Oss` arms; leave other arms intact) to
> minimize the diff against upstream.

## Group 4 — DockTilePlugin (macOS, Tarp-owned)

`app/DockTilePlugin/`:
- `Info.plist`: `CFBundleIdentifier`/`CFBundleExecutable`/`CFBundleName`/
  `NSPrincipalClass` all = `WarpDockTilePlugin` / `dev.warp.WarpDockTilePlugin`.
- `WarpDockTilePlugin.h` / `.m`: ObjC class `WarpDockTilePlugIn` (rename optional —
  internal symbol, not user-facing; renaming the **files** ripples into build.rs).
- `Makefile`: `BUNDLE_NAME = WarpDockTilePlugin.docktileplugin`, `OBJC_FILES = WarpDockTilePlugin.m`.
- `README.md`: references the plugin name.
- `Resources/*.png` — dock-tile icon variants (`original.png`, `warp_2.png`, …);
  brand art, replace with Tarp art or trim to one.

Build/bundle wiring (must stay in lockstep with the above):
- `app/build.rs:48-51, 56, 67-68` — rerun-if-changed + builds
  `WarpDockTilePlugin.docktileplugin`.
- `script/macos/bundle:320` `DOCK_TILE_PLUGIN_DIR=…/WarpDockTilePlugin.docktileplugin`;
  `:518` log string; `:523-524` `plutil -insert NSDockTilePlugIn … WarpDockTilePlugin.docktileplugin`.

**Action:** the *identifier* `dev.warp.WarpDockTilePlugin` → `dev.tarp.…`. The
ObjC class/file names can stay `WarpDockTilePlugin` (internal) to reduce churn —
your call. If renamed, update `Info.plist`, `Makefile`, `build.rs`, `script/macos/bundle`
together.

## Group 5 — Linux `.desktop` files + channel dirs

`app/channels/{oss,stable,dev,local,preview}/dev.warp.*.desktop` (one per channel):

OSS one (`app/channels/oss/dev.warp.WarpOss.desktop`) — keep, rewrite, and rename
the file:
- filename `dev.warp.WarpOss.desktop` → `dev.tarp.Tarp.desktop`.
- `Name=WarpOss` → `Tarp`.
- `Exec=warp-terminal-oss %U` → `tarp %U`.
- `StartupWMClass=dev.warp.WarpOss` → `dev.tarp.Tarp`.
- `Icon=dev.warp.WarpOss` → `dev.tarp.Tarp`.
- `MimeType=x-scheme-handler/warposs;` → `…/tarp;`.

Delete the other four channel dirs entirely (`app/channels/{stable,dev,local,preview}/`)
— each has a `.desktop` + an `icon/` tree.

> Note the `.desktop` `Exec` is `warp-terminal-oss` (the Linux *installed* binary
> name), distinct from the cargo bin `warp-oss`. The Linux name comes from the
> packaging templates (Group 8), so keep these consistent: pick `tarp`.

## Group 6 — Icons (per-channel art)

`app/channels/<channel>/icon/`:
- `no-padding/{16,32,48,64,128,256,512}.png` + `icon.ico` (the bundled app icon).
- `AppIcon.icon/` (macOS `.icon` package): `icon.json`, `Assets/*.png|*.svg`.
  Brand glyph art lives here, e.g.:
  - `app/channels/stable/icon/AppIcon.icon/Assets/Glyph (1).svg`
  - `app/channels/local/icon/AppIcon.icon/Assets/warp-glyph 3.svg`
  - `app/channels/dev/icon/AppIcon.icon/Assets/Glyph.svg`

Only `app/channels/oss/icon/` needs Tarp art (`512x512.png`, `icon.ico` — oss is
the minimal set; it lacks the AppIcon.icon package and small sizes the others have,
so you may want to generate a full set). Regen via `script/compile_icon`. Delete
the other channels' icon trees with the channel collapse.

## Group 7 — `images/` and bundled SVG/PNG brand assets

`images/`:
- `images/Built-With-Warp-Export@2x.png` — "Built With Warp" badge; remove (or
  replace with a Tarp badge).
- `images/Powered-By-Oz-Export@2x.png` — Oz (ambient agents) badge; remove (cloud/AI, A2/A4).

`app/assets/` warp-named brand art (40 files; the logos/loaders are pure branding):
- `app/assets/bundled/svg/warp.svg`, `warp-2.svg`, `warp-3.svg`,
  `warp-logo-{neutral,light,dark}.svg`, `warp-logo-with-{dark,light}-title.svg`,
  `warp-drive.svg`, `warp-loading-0..11.svg` (12 loader frames) — replace/remove.
- `app/assets/resources/mac/warp_install_image.png` — DMG background.
- `app/DockTilePlugin/Resources/warp_2.png` (see Group 4).
- Onboarding art (`app/assets/async/png/onboarding/openwarp_launch_banner.png`,
  `…/customize_warpdrive_*.png`) — disappears with onboarding removal (A2/A5); no
  hand-edit needed.

These files are referenced by name from Rust (asset loaders). Renaming the files
forces matching edits in `app/src` load sites; **easier to keep the filenames and
just swap the file contents** (drop in Tarp art under the same name) to avoid
touching load code. Flag any that are referenced from tracked crates (a `warpui`
asset map) before renaming.

## Group 8 — Linux packaging templates (`resources/linux/`)

These bake "warp" into the Linux package/binary name and metadata:
- `resources/linux/arch/app/warp.sh.template` (rename file): launcher; reads
  `warp-terminal@@CHANNEL_SUFFIX@@-flags.conf` (`:6-7`), execs
  `/opt/warpdotdev/warp-terminal…/@@BINARY_NAME@@` (`:11`).
- `resources/linux/rpm/app/warp.spec.template` / `cli/warp.spec.template`:
  `Name: warp-terminal…` (`:6`), `Summary`/`%description` "Warp, the Rust-based
  terminal… AI built in" (`:5, :31-33`), `Url: https://warp.dev` (`:10`),
  `Packager … @warp.dev` (`:11`), `License: https://warp.dev/terms-of-service`
  (`:12`), `%{prefix}/warpdotdev/%{name}` (`:52,:56,:64`), apt/yum repo URLs
  `releases.warp.dev/linux/…` (`:163-194`).
- `resources/linux/debian/app/control.template` + `cli/control.template`:
  `Package: warp-terminal…` (`:1`), `Maintainer … @warp.dev` (`:8`),
  `Homepage: https://warp.dev/` (`:9`), `Description: Warp, the Rust-based…` (`:10-13`).
- `resources/linux/debian/{app,cli}/{postinst,postrm}.template`,
  `common/{postinst,postrm}.repo.template` — repo setup referencing `releases.warp.dev`.
- `resources/linux/arch/{app,cli}/PKGBUILD.template`.

**Action:** package name `warp-terminal` → `tarp`; drop the AI marketing copy and
the `releases.warp.dev` repo wiring (Tarp has no release server — A7 territory).

## Group 9 — The Warp glyph font patch (trademark)

`script/patch_font_with_warp_glyph` (fontforge script) + `script/warp.svg` (the
logo SVG it imports):
- `DEFAULT_SVG_PATH = …/warp.svg` (`:14`), `DEFAULT_GLYPH_NAME = "warpLogo"` (`:16`),
  codepoint `0xE500` (PUA) (`:15`).
- Embeds the Warp logo into a font at U+E500.

**Action (trademark):** the Warp logo is trademarked — **remove `script/warp.svg`
and either delete the patch step or repoint it at a Tarp glyph** (rename the script
to `patch_font_with_tarp_glyph`, `--svg tarp.svg`, `--name tarpLogo`). Check where
U+E500 is rendered (the terminal may print the brand glyph in the prompt/UI);
search Rust for `` / `0xE500` before deciding whether Tarp needs its own
glyph at the same codepoint.

## Group 10 — `about.hbs` / `about.toml` (license attribution)

- `about.hbs`: header text `Third-Party Licenses for Warp` and `…included in the
  Warp distribution.` (lines 1, 4). → "Tarp". Pure cosmetic; safe.
- `about.toml`: cargo-about config. The comment block (`# These are Warp's own
  private crates … warpdotdev …`) is descriptive; the `private`/`accepted` lists
  are functional. No brand string is load-bearing here — A4 owns the license list;
  leave the functional parts, optionally update the comment.

## Group 11 — User-facing "Warp" strings in app/src (Tarp-owned)

The product-name strings the user actually sees. (Display in some tool output was
glitched to `ln`; the `file:line` refs and quoted strings below were read
directly and are exact.)

- **Window title:** `app/src/root_view.rs:108` `const WINDOW_TITLE: &str = "Warp";`
  → `"Tarp"`. Also literal `"Warp"` window-title fallbacks at
  `root_view.rs:703, 746, 798, 1170, 1355`.
- **macOS app menu:** `app/src/app_menus.rs:247` `Menu::new("Warp", menu_items)`
  → `"Tarp"` (the application menu title). `:142` `CustomAction::ShowAboutWarp`
  (the About item) — the *action* enum name is internal; the rendered "About Warp"
  label is derived from the app name. `:928` link `"Warp Documentation..."` →
  "Tarp Documentation".
- **About box:** `CustomAction::ShowAboutWarp` wired at
  `app/src/util/bindings.rs`, `app/src/workspace/mod.rs`, `app/src/app_menus.rs:142`.
  Rename the visible "About Warp" label; the enum variant can stay.
- **Quit warning:** `app/src/quit_warning/mod.rs` "Quit Warp?" → "Quit Tarp?".
- **"Warp on Web" home:** `app/src/workspace/home.rs` `WARP_HOME_TITLE = "Warp on
  Web"` + body copy — this is a cloud/web feature; removed by A2, no hand-edit.
- **Auth view:** `app/src/auth/auth_view_body.rs` "Welcome to Warp!" — removed
  with accounts (A5), no hand-edit.
- **Update button:** `app/src/workspace/view.rs` (comment notes the word "Warp" is
  used in the Update-Ready button to make it obvious it's Warp) → "Tarp".
- **SSH menu item:** `app/src/app_menus.rs:56` "Hide Warpified SSH Blocks" — see
  the Warpify note below.

Rough magnitude: ~713 `app/src` files contain a quoted string with the word
`Warp`, but the **large majority** are inside removed surfaces (AI/cloud/onboarding/
auth) and vanish for free. The hand-edit set is the small list above plus the
menus/settings tree. Do this pass **after** the AI/cloud/account deletions (A1–A5)
so you're not editing strings you're about to delete.

### "Warpify" — feature name, not pure branding (831 refs)

`warpify`/`Warpify` appears 831× in `app/src` and across
`app/assets/bundled/ssh/*` (e.g. `warpify_ssh_session.sh`,
`install_tmux_and_warpify_*.sh`). This is the **SSH subshell-bootstrap feature**
(injecting shell integration into remote sessions), a *terminal* feature that
stays. The token "warp" is baked into the feature name, env vars, and asset
filenames. **Recommendation: do NOT rename Warpify** — it's a pervasive internal
identifier; renaming it is large and risky for zero user benefit. Optionally
rename only the user-visible menu label ("Hide Warpified SSH Blocks").

## URLs — `warp.dev` / `docs.warp.dev` (separate list, as requested)

Centralized constants (the high-value single point):
- `app/src/util/links.rs`:
  - `:3` `USER_DOCS_URL = "https://docs.warp.dev/"`
  - `:5` `GITHUB_ISSUES_URL = "https://github.com/warpdotdev/Warp/issues"`
  - `:6` `SLACK_URL = "http://go.warp.dev/join-preview"`
  - `:7` `PRIVACY_POLICY_URL = "https://www.warp.dev/privacy"`
  - `:10` `feedback_form_url()` → `https://github.com/warpdotdev/Warp/issues/new/choose`
    with a `warp-version` query param (`:13`).

  → repoint to Tarp's repo/docs (or remove the menu items if Tarp has none).

Magnitude (whole tree, `crates/` + `app/src`):
- **400** files reference `warp.dev`; **145** reference `docs.warp.dev`.
- The overwhelming majority of `docs.warp.dev` hits are in `app/src/ai/**`,
  `app/src/cloud_object/**`, agent/onboarding views — they **delete for free**
  with A1/A2/A5. Examples observed: `app/src/ai/agent_tips.rs`,
  `app/src/ai/agent_management/cloud_setup_guide_view.rs`,
  `app/src/drive/import/modal_body.rs`.
- Surviving (terminal-core) URL hits to fix by hand:
  `app/src/util/links.rs` (above), `app/src/workspace/view.rs` (Wayland/linux help
  links), `app/src/resource_center/*` (keybindings/changelog docs links),
  `crates/warpui/src/rendering/wgpu/resources.rs` (a `docs.warp.dev#linux` GPU help
  link — ⚠ tracked crate), `crates/warp_cli/src/{lib,share}.rs` (CLI help text;
  `share.rs` is cloud, likely removed).
- Cloud config URLs in `crates/warp_core/src/channel/config.rs` (`app.warp.dev`,
  `rtc.app.warp.dev`, `sessions.app.warp.dev`, `oz.warp.dev`) — A2/A5 (⚠ tracked).

**Approach:** fix the `links.rs` constants and the handful of surviving terminal
links; let everything under removed surfaces go with the deletions; treat the
tracked-crate URL hits (`warpui`, `warp_core`) as their own small isolated commits.

## Shell-integration markers / env vars (`app/assets/bundled/bootstrap/*`)

These files are **tracked-from-upstream** (08-upstream-sync lists `bootstrap/*` as
tracked). They encode "warp" in two distinct ways:

### (a) `WARP_*` environment variables — the terminal↔shell contract (76 distinct)

The bootstrap scripts (`bash_body.sh`, `zsh_body.sh`, `fish.sh`, `pwsh.ps1`,
`*_init_shell.*`, `*_init_subshell.*`) read/write **76 distinct `WARP_*` env
vars**, e.g. `WARP_SESSION_ID`, `WARP_BOOTSTRAPPED`, `WARP_CLIENT_VERSION`,
`WARP_HONOR_PS1`, `WARP_IS_SUBSHELL`, `WARP_GENERATOR_PIDS*`, `WARP_TMP_DIR`,
`WARP_USE_SSH_WRAPPER`, … (full set enumerable via
`rg -No -i 'WARP_[A-Za-z_]+' app/assets/bundled/bootstrap/ | sort -u`).

These are **set on the Rust side** in Tarp-owned code:
- `app/src/terminal/local_tty/unix.rs:275,279,323,325,774,776,797`
  (`WARP_CLIENT_VERSION`, `WARP_HONOR_PS1`).
- `app/src/terminal/bootstrap.rs:234,256` (`WARP_BOOTSTRAPPED`, `WARP_HONOR_PS1`).
- plus the Windows path (`app/src/terminal/local_tty/windows/environment.rs`).

**Recommendation: do NOT rename the `WARP_*` env vars.** They are an internal
contract between the Rust set-side (Tarp-owned) and the bootstrap consume-side
(tracked). Renaming would:
1. force coordinated edits across **both** sides of the contract, and
2. churn every tracked bootstrap script — exactly the merge-conflict surface
   08-upstream-sync says to protect.

They are not user-visible (env var names in a subshell). Leave them as `WARP_*`.
If a hard requirement to scrub the name exists, do it as a single mechanical
rename across set-side + scripts in one commit, and accept the sync cost.

### (b) `warp` strings that ARE worth touching in bootstrap

- Issue-link comments: `app/assets/bundled/bootstrap/bash_body.sh` and
  `zsh_body.sh` reference `github.com/warpdotdev/Warp/issues/{1262,2636,11520}`;
  `bash.sh`/`bash_body.sh` reference `linear.app/…/WAR-2592`, `WAR-6064`. Comments
  only — update or drop (low priority, no behavior change).
- apt source filename `warp.list` / dist-upgrade handling in `fish.sh`,
  `bash_body.sh`, `zsh_body.sh` (`warp.list.distUpgrade` → `warp.list`). This is
  tied to the Linux **package name** (`warp-terminal`, Group 8). If you rename the
  package to `tarp`, this apt-source filename must match — coordinate Group 8 +
  these scripts. ⚠ tracked scripts.
- `WARP_BOOTSTRAP_VAR` sentinel (`bash.sh:443`, `zsh.sh:19`) — internal marker,
  leave (see (a)).

### macOS bundle script app-name map

`script/macos/bundle:272-312` maps channel → `WARP_BIN` + `WARP_APP_NAME`:
`warp`/`WarpLocal`, `dev`/`WarpDev`, `preview`/`WarpPreview`, `stable`/`Warp`,
`warp-oss`/`WarpOss`. With the single-channel collapse, reduce to one entry:
`WARP_BIN=tarp`, `WARP_APP_NAME=Tarp`. Also `:320-524` use `WARP_APP_NAME`/
`WARP_BIN`/`BUNDLE_ID` to assemble `$NAME.app`, the dock-tile plugin, and
`New $WARP_APP_NAME Tab/Window Here` services menu strings (`:423, :442`). (A7
owns the release scripts; this is the branding overlap.)

---

## Sequencing (do branding LAST within each subsystem)

1. **A1–A5 first.** Delete AI/cloud/account/telemetry/onboarding. This vaporizes
   the bulk of "Warp" strings, `docs.warp.dev` links, Sentry frameworks, and the
   `app.warp.dev`/`oz.warp.dev` config — for free, with no branding edits.
2. **Channel collapse** (Group 1, 2, 5, 6, + the bundle script map): delete the
   four non-Oss bin targets, bundle blocks, channel dirs, icon trees, `.desktop`
   files. Decide enum-keep vs enum-trim for `warp_core` (Group 3).
3. **OSS plist rename** (`app/src/bin/oss.rs`, Group 2) — the one entrypoint.
4. **Centralized strings/URLs:** `app/src/util/links.rs`, `root_view.rs`
   `WINDOW_TITLE`, `app_menus.rs` menu title + About + docs link (Group 11 + URLs).
5. **Assets:** swap art in place under existing filenames (Groups 6, 7); regen
   icons via `script/compile_icon`. Remove the Warp glyph (`script/warp.svg`) and
   re-point/rename `patch_font_with_warp_glyph` (Group 9).
6. **Linux packaging** templates (Group 8) + the matching apt-source name in the
   bootstrap scripts (shell section (b)).
7. **DockTilePlugin** identifier (Group 4) — keep ObjC class/file names to limit
   churn unless a clean scrub is required.
8. **Tracked-crate touch-ups** (`warp_core` channel strings/paths, the `warpui`
   GPU help URL) — each as its own small, isolated commit for sync hygiene.
9. **Leave `WARP_*` env vars and the Warpify feature name alone** (shell section
   (a) + the Warpify note).
10. **Verify the built artifact**, not just source: the `.app`'s `Info.plist`,
    bundle id, dock-tile, `.desktop`, and data dir (`~/Library/Application
    Support/…`). Several brand surfaces are assembled at bundle time
    (`script/macos/bundle`, `plutil` inserts, `compile_icon`).

## Risks & tracked-crate touchpoints (merge-conflict flags)

- ⚠ `crates/warp_core/src/channel/{mod.rs,state.rs,config.rs}` and `paths.rs` +
  `paths_tests.rs` — channel enum, `cli_command_name`, `url_scheme`, the OSS
  `AppId` fallback, the data-dir name mapping, and the cloud-config URLs. Tracked
  crate; isolate edits, prefer single-channel (touch only `Oss` arms).
- ⚠ `crates/warpui/src/rendering/wgpu/resources.rs` — a `docs.warp.dev` Linux GPU
  help URL inside a tracked renderer crate.
- ⚠ `app/assets/bundled/bootstrap/*` — tracked shell scripts; the `WARP_*` env
  contract and apt-source name live here. Renaming env vars or the package name
  forces churn in tracked files. Recommendation: don't rename env vars.
- **Crate name vs product name:** the Rust crate is named `warp` (`app/Cargo.toml`
  `:7,:13`, `warp::run()`). Renaming the crate touches every `warp::`/`use warp`
  path and is gratuitous. Rename the **product/binary/bundle**, not the crate.
- **Data-dir migration:** changing the app name changes the user-data path; existing
  Warp-OSS users (if any) won't see old data. Acceptable for a fresh fork; note it.
- **Trademark:** the Warp logo (`script/warp.svg`, the `warp-logo-*.svg` set, dock
  art, icons) and the brand glyph at U+E500 must be removed/replaced, not just
  renamed — Warp's marks are trademarked even though the code is AGPL.
- **Built-at-bundle surfaces:** grepping source misses brand strings injected by
  `script/macos/bundle` (`plutil` inserts: services-menu "New Warp Tab Here",
  dock-tile plugin) and by the channel-config generator. Verify the artifact.
