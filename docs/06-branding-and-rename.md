# 06 — Branding & Rename (Warp → Tarp)

Inventory of Warp branding and identity to change for Tarp. See
[`../tarp.md`](../tarp.md) for the name rationale and logo direction.

## Branding footprint

- **220 files** reference `warp.dev` / `warpdotdev` (176 of them `.rs`).
- User-facing "Warp" strings appear in settings, about screen, window titles,
  notifications, and shell integration.

## Bundle identity (`app/Cargo.toml`)

There are **5 bundle targets / channels**, each with its own identifier, name, and icon:

| Bin target | Identifier | App name | Notes |
|---|---|---|---|
| `warp-oss` | `dev.warp.WarpOss` | `WarpOss` | **The OSS build** — this is what `./script/run` builds without `warp-channel-config`. Our default base. |
| `stable` | `dev.warp.Warp-Stable` | `Warp` | Bundles Sentry framework. |
| `preview` | `dev.warp.Warp-Preview` | `WarpPreview` | Bundles Sentry. |
| `dev` | `dev.warp.Warp-Dev` | `WarpDev` | Bundles Sentry. |
| `warp` | `dev.warp.Warp-Local` | `WarpLocal` | Local internal build. |

- `copyright = "© 2025, Denver Technologies, Inc"` on each.
- `category = "public.app-category.developer-tools"`.
- App version is `0.1.0` (channel/version is otherwise driven by the
  `channel_versions` crate + release workflows, not a static field).
- `DockTilePlugin/Info.plist` → `dev.warp.WarpDockTilePlugin`.
- Linux `.desktop` files per channel under `app/channels/*/` (e.g.
  `dev.warp.Warp.desktop`, `StartupWMClass=dev.warp.Warp`, `Icon=dev.warp.Warp`).

## Rename checklist

### Identifiers & app names
- [ ] Decide the Tarp identifier namespace (e.g. `dev.tarp.Tarp` / `com.tarp.*`).
- [ ] Collapse the 5 channels to what Tarp actually ships (likely just one — see
      [`07-ci-and-release.md`](07-ci-and-release.md)); rename its identifier/name.
- [ ] Binary name `warp-oss`/`warp` → `tarp` (assess shell-integration coupling).
- [ ] `DockTilePlugin` identifier.
- [ ] `.desktop` files: filename, `StartupWMClass`, `Icon`, `Name`, `Exec`.
- [ ] `copyright` → add Tarp; the AGPL requires keeping Denver Technologies' notice
      (see [`04-licensing.md`](04-licensing.md)).

### Icons & visual assets
- [ ] `script/compile_icon` + `app/channels/*/icon/` (512x512 png, .ico).
- [ ] `images/`, `resources/` — replace Warp logos with the Tarp logo (literal tarp).
- [ ] `script/patch_font_with_warp_glyph` — **remove the Warp glyph** (trademark);
      decide if Tarp needs its own glyph.

### Shell integration (`app/assets/bundled/bootstrap/`)
The terminal injects shell bootstrap for bash/zsh/fish/pwsh:
`bash_body.sh`, `zsh_body.sh`, `fish.sh`, `pwsh.ps1`, `*_init_shell.*`,
`*_init_subshell.*`, plus `bash-preexec-LICENSE.md`.
- [ ] Audit these for `warp`-prefixed env vars, markers, and URLs (e.g. issue links
      like `github.com/warpdotdev/warp/issues/...`). These set the terminal↔shell
      contract; rename carefully and keep behavior identical.

### User-facing strings
- [ ] Settings UI, About screen (`about.hbs`, `about.toml`), window title, menus.
- [ ] Notification/copy strings that say "Warp".
- [ ] `docs.warp.dev` / `warp.dev` URLs in help/links → Tarp docs (or remove).

### Repo-level docs (also branding)
- [ ] `README.md`, `WARP.md` (→ `TARP.md`?), `FAQ.md`, `CONTRIBUTING.md`,
      `CODE_OF_CONDUCT.md`, `SECURITY.md`, issue/PR templates.

## Risk note
Branding is spread across **code, assets, plists, desktop files, and shell
scripts**. Verify the **built artifact** (the `.app`/binary), not just source
strings — some brand surfaces are generated at bundle time
(`update_plist`, `compile_icon`, `prepare_bundled_resources`).
