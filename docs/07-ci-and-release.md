# 07 — CI & Release

What CI/automation exists, what's reusable for Tarp, and what to cut.

## CI workflows (`.github/workflows/`) — 20 workflows

Verdict legend: **Keep/slim** · **Cut** (Warp-internal automation) · **Replace**.

| Workflow | Purpose | Verdict |
|---|---|---|
| `ci.yml` (37 KB) | Main CI: `tests`, `remote-server-tests`, `database-migration`, `lints`, `general-lint`, `wasm-lint`, `check-release-compilation`, `ci-result`. | **Keep/slim** — drop remote-server/db/wasm jobs once those features go; keep fmt+clippy+tests. |
| `create_release.yml` (83 KB) | Full multi-platform release build/sign/publish. | **Replace** with a lean Tarp release workflow (reuse the bundle scripts). |
| `cut_new_releases.yml`, `cut_new_release_candidate.yml`, `delete_release.yml` | Warp's channel/RC release cadence. | **Cut/Replace** — Tarp uses simple semver tags. |
| `populate_build_cache.yml` | Warp's build cache. | **Cut** (or re-point to our cache). |
| `repo-sync.yml` | Calls `warpdotdev/repo-sync` to sync internal↔public. Triggers on `master`. | **Cut** — Warp-internal; also references `master` (branch rename). |
| `changelog_draft.yml` | Oz-driven changelog generation. | **Cut** (or replace with a simple changelog). |
| `check_approvals.yml`, `sync-pr-checks.yml` | Warp PR gating. | **Cut/slim**. |
| `close_stale_fix_prs.yml`, `stale_requested_changes_prs.yml`, `warp_cleanup_fix_prs.yml` | Stale-PR bots for Warp's agent PR flow. | **Cut**. |
| `label_external_contributors.yml` | Labels external PRs. | **Optional keep**. |
| `docubot_reply_to_comment.yml` | Warp Docubot. | **Cut**. |
| `feature_flag_cleanup.yml` | Automates feature-flag cleanup. | **Cut** (we'll do flag surgery manually). |
| `update-dedupe-local.yml`, `update-pr-review-local.yml`, `update-triage-local.yml` | Sync Warp's agent skills into the repo. | **Cut**. |

Also under `.github/`: `actions/`, `dependabot.yml`, `ISSUE_TEMPLATE/`,
`PULL_REQUEST_TEMPLATE/`, `STAKEHOLDERS`, `issue-triage/`, `scripts/` — most are
Warp-org specific; review and replace/trim.

**Net:** the vast majority of CI is Warp-internal agent/release automation to
**delete**. Tarp needs ~2 workflows: a slim PR CI (fmt/clippy/test) and a tag-driven
release.

## Packaging — strong, reusable

Warp's bundling scripts are mature and are the **biggest asset to keep**:

### macOS (`script/macos/`, `script/run`)
- Builds a real `.app` via `cargo bundle`, `update_plist`, `compile_icon`,
  `prepare_bundled_resources`, codesign with entitlements.
- Output: `target/<profile>/bundle/osx/<Name>.app`.

### Linux (`script/linux/`)
- `bundle` (303 lines) orchestrates: `bundle_deb`, `bundle_rpm`, `bundle_arch`,
  `bundle_appimage`, `bundle_install`. Covers **.deb / .rpm / Arch / AppImage**.
- `install_linuxdeploy`, `linuxdeploy-plugin-warp` (rename), `sign_arch_packages`.

### Windows (`script/windows/`)
- `bundle.ps1`, Inno Setup (`windows-installer.iss`, `environment.iss`),
  `prepare_bundled_resources.ps1`, installer images.

## Release plan for Tarp (proposed)

0. **Cadence philosophy — stability first.** Tarp optimizes for *fewer, stable*
   releases. Expect a **burst of releases early** (during de-Warp + stabilization),
   then deliberately **slow to ~monthly** once stable, slowing further over time.
   Releases are batched, not continuous; most upstream cherry-picks land between
   releases and ship together. This favors a tag-driven release (below) over any
   auto-cut cadence.
1. **Versioning:** drop Warp's channel/`channel_versions` model; use semver git
   tags `vX.Y.Z`.
2. **One release workflow:** on tag push → matrix build (mac/linux/win) → run the
   existing bundle scripts → attach artifacts to a GitHub Release.
3. **Signing/notarization:** decide per platform. v1 can ship unsigned + clear
   install docs (Gatekeeper/SmartScreen caveats); add signing later.
4. **Updater:** Warp's autoupdate is cloud-tied (`autoupdate*` features,
   `autoupdate_ui_revamp`). Remove it; document manual update for v1, or add a
   minimal GitHub-Releases-based self-update later.
5. **Attribution:** ship the regenerated `THIRD_PARTY_LICENSES` (cargo-about) in
   the bundle (see [`04-licensing.md`](04-licensing.md)).
6. **Channels:** collapse the 5 bundle targets to one (`tarp`), or keep
   stable+preview only (see [`06-branding-and-rename.md`](06-branding-and-rename.md)).

## Branch rename
Several workflows key off `master` (e.g. `repo-sync.yml`). When renaming
`master → main`, update or (mostly) delete these workflows.
