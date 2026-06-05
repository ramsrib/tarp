# A7 — CI / Release Plan (de-Warp)

Scope: the `.github/` automation surface and the `script/` packaging surface, and
the two replacement workflows Tarp needs. This is an **execution-ready** plan: it
classifies every existing workflow keep/slim/cut/replace with a reason, then
drafts (as illustrative YAML inside this doc — **do NOT create or enable real
workflow files from this analysis pass**) a slim PR CI and a tag-driven release.

This builds on and supersedes the summary in
[`../07-ci-and-release.md`](../07-ci-and-release.md). Where that doc gives the
verdict table, this doc gives the file:line evidence, the exact bundle invocation,
and the YAML drafts.

---

## 0. Key facts that drive the whole plan

These were confirmed by reading the scripts; they make Tarp's CI cheap.

1. **An `oss` channel already exists in all three bundle scripts** and is already
   AI/cloud-tolerant where it matters most — it drops Sentry/crash reporting:
   - macOS: `script/macos/bundle:309-316` → `WARP_BIN=warp-oss`, `WARP_SCHEME_NAME=warposs`,
     `FEATURES="release_bundle,extern_plist"` (no `cocoa_sentry`).
   - Linux: `script/linux/bundle:175-182` → `WARP_BIN=warp-oss`, `APP_NAME=WarpOss`,
     `FEATURES="release_bundle"` (no `crash_reporting`).
   - Windows: `script/windows/bundle.ps1:109-115` → `$WARP_BIN='warp-oss'`,
     `$FEATURES='release_bundle,gui'` (no `crash_reporting`).
   - The umbrella dispatcher `script/bundle:1-33` routes by `uname -s` to the
     per-OS script, so `./script/bundle --channel oss ...` is the single entry
     point on every platform.
   > Implication: Tarp's release workflow is largely "call the existing `oss`
   > bundle path." The `oss` channel is the seam Tarp keeps and hardens; over
   > time the `local/dev/preview/stable` branches in those scripts become dead
   > and can be deleted (a `script/`-owned change, not a tracked-crate change).

2. **CI already knows how to do an OSS release-compilation check.**
   `ci.yml:789-801` runs `./script/bundle --channel oss --nouniversal --check-only`
   (and the wasm variant). `--check-only` runs `cargo check` with the exact
   release feature set (`script/linux/bundle:220-222`). This is the cheapest
   "does the OSS build compile" gate and Tarp should keep it.

3. **`presubmit` is the local mirror of CI** (`script/presubmit`): fmt (`./script/format --check`),
   inline-test-module check, clippy (`--workspace --exclude warp_completer` then
   `-p warp_completer`), clang-format, wgslfmt, nextest
   (`--workspace --exclude command-signatures-v2`). Tarp's slim PR CI should be a
   thin GitHub wrapper over the same commands so local and CI stay in sync.

4. **License attribution is already wired into bundling.**
   `script/install_cargo_release_deps:28` installs `cargo-about@0.8.4`;
   `script/prepare_bundled_resources:105-110` emits `THIRD_PARTY_LICENSES.txt`
   via `cargo about generate -c about.toml`. The release workflow gets attribution
   "for free" as long as it passes `install_release_deps: true` to
   `prepare_environment` (`.github/actions/prepare_environment/action.yml:14-15,112-133`).

5. **Almost everything keys off `master`** — `repo-sync.yml:4`, the `params`
   gating in `ci.yml:54-55`, `cut_new_releases.yml:39` ("Check for master branch"),
   `populate_build_cache.yml:6`, the rust-cache `save-if` in
   `prepare_environment/action.yml:80` and `ci.yml:663`. The master→main rename
   touches all of these (see §5).

---

## 1. Workflow-by-workflow verdicts (`.github/workflows/`, 20 files)

Verdict legend: **KEEP** · **SLIM** (keep but strip Warp-internal jobs) ·
**CUT** (delete; Warp-internal) · **REPLACE** (delete and supersede with a Tarp
draft below).

| File | Size | Purpose (evidence) | Verdict | Reason |
|---|---|---|---|---|
| `ci.yml` | 37 KB | Main CI: jobs `params`, `tests`, `remote-server-tests`, `database-migration`, `lints`, `general-lint`, `wasm-lint`, `check-release-compilation`, `ci-result`. | **REPLACE** (with slim PR CI, §3) | Most jobs are for surfaces Tarp removes — see per-job breakdown §2. Net keep ≈ fmt + clippy + tests + oss check-compile. Rebuilding clean is less work than surgically editing a 858-line file, and avoids carrying Warp infra refs (GCP WIF `ci.yml:231`, trunk.io uploads `ci.yml:272-277`, Namespace runners `ci.yml:76`). |
| `create_release.yml` | 83 KB | Full multi-platform sign/notarize/publish across all channels; `workflow_dispatch` + repo-sync-style triggers (`create_release.yml:11-40`). | **REPLACE** (with tag release, §4) | Tied to channels, `channel_versions`, Sentry release upload, GCS, signing secrets. Tarp wants tag→build→attach. Reuses the same bundle scripts underneath. |
| `cut_new_releases.yml` | 3.3 KB | Cron `0 8 * * *` auto-cut of dev/preview/stable; checks `master` (`:39`). | **CUT** | Warp channel cadence. Tarp is tag-driven + stability-first; no auto-cut. |
| `cut_new_release_candidate.yml` | 2.0 KB | Cuts RC, writes `warpdotdev/channel-versions` (`:51`). | **CUT** | RC/channel model dropped; depends on a Warp-private repo. |
| `delete_release.yml` | 4.8 KB | Deletes a release branch, edits `channel_versions.json` in `warpdotdev/channel-versions` (`:68`). | **CUT** | Channel model + private repo dependency. Manual `gh release delete` suffices. |
| `populate_build_cache.yml` | 0.7 KB | Cron + on push to `master` (`:6,:16-17`) to warm Warp's build cache. | **CUT** | Warp-internal cache. Rebuild on Tarp's own cache later if cold builds hurt. |
| `repo-sync.yml` | 2.5 KB | Calls `warpdotdev/repo-sync/*` to sync internal↔public; triggers on `master` (`:4`); restack/approve/escalation jobs reference `warpdotdev/warp-internal` (`:24,:43,:59`). | **CUT** | Pure Warp internal↔public mirroring. Irrelevant and references private repos + `master`. |
| `changelog_draft.yml` | 3.0 KB | Oz/Namespace-driven changelog for `stable/preview/dev` tags (`:9,:37 runs-on: namespace-profile-ubuntu-small`). | **CUT** (replace later with hand-written `CHANGELOG.md`) | Channel-tag shaped and runs on Namespace runners. Tarp's stability-first cadence makes a manual changelog cheap (the `changelog-draft` skill can still be invoked locally). |
| `check_approvals.yml` | 5.8 KB | Warp PR approval gating. | **CUT** | Warp org review policy; GitHub branch-protection covers Tarp. |
| `sync-pr-checks.yml` | 1.4 KB | Syncs PR check state (`runs-on: ubuntu-slim`). | **CUT** | Warp PR plumbing tied to repo-sync. |
| `close_stale_fix_prs.yml` | 2.9 KB | Cron `0 12 * * *`; closes `oz-agent-fix/run-*` branches (`:42`). | **CUT** | Warp's Oz agent-PR janitor. No Oz in Tarp. |
| `stale_requested_changes_prs.yml` | 2.2 KB | Cron daily; nags stale changes-requested PRs. | **CUT** | Warp PR-flow bot. Optional: re-add a generic `actions/stale` later if needed. |
| `warp_cleanup_fix_prs.yml` | 2.6 KB | Cleans up Warp "fix PRs". | **CUT** | Warp agent-PR flow. |
| `label_external_contributors.yml` | 2.4 KB | Labels external-contributor PRs. | **CUT** (optional KEEP) | Mild value if Tarp takes OSS PRs, but encodes Warp org membership; defer. |
| `docubot_reply_to_comment.yml` | 2.3 KB | Runs Warp Docubot on `@docubot` mention; uses `.github/actions/docubot` + a release channel/profile (`:22,:30`). | **CUT** | Warp AI doc bot. |
| `feature_flag_cleanup.yml` | 12.8 KB | Cron `0 15 * * *`; uses `warpdotdev/oz-agent-action@main` (`:38`) to open `oz-agent/cleanup-feature-flag-*` PRs (`:103`). | **CUT** | Warp Oz automation. Tarp does flag surgery manually per `05-removal-map.md`. |
| `update-dedupe-local.yml` | 0.7 KB | Cron Mon 09:30; calls `warpdotdev/oz-for-oss/.github/workflows/update-dedupe.yml@main` (`:17`). | **CUT** | Syncs Warp agent skill into repo. |
| `update-pr-review-local.yml` | 0.7 KB | Cron Mon 09:00; `warpdotdev/oz-for-oss/.../update-pr-review.yml@main` (`:17`). | **CUT** | Warp agent-skill sync. |
| `update-triage-local.yml` | 0.7 KB | Cron Mon 09:15; `warpdotdev/oz-for-oss/.../update-triage.yml@main` (`:17`). | **CUT** | Warp agent-skill sync. |
| `README.md` + `release_configurations.json` | — | Doc + channel config schema (nightly/weekly, Sentry project, Slack channel). | **CUT** | Describes the Warp channel/Sentry/Slack release model Tarp drops. |

**Net of the 20 workflows: 2 REPLACE, 16 CUT, 0 plain KEEP.** Tarp ends with the
two drafted workflows below. (`ci.yml`/`create_release.yml` are "replace" rather
than "slim" because the editing cost and Warp-infra coupling exceed a clean
rewrite — but the *bundle scripts they call* are reused intact.)

### Supporting `.github/` assets

| Path | Verdict | Reason |
|---|---|---|
| `.github/actions/prepare_environment/` | **SLIM → KEEP** | The release draft (§4) reuses it. Slim out: GCP `setup-xcode`/channel-config SSH for `warp-internal` (`action.yml:96-102`), Namespace cache (`:82-88`), and the `save-if: refs/heads/master` (`:80` → `main`). The protoc/node/rust-cache/LFS steps stay. This is `.github`-owned, so editing it is safe (not a tracked Rust crate). |
| `.github/actions/get_channel_config/` | **CUT** | Channel-config model dropped. |
| `.github/actions/docubot/` | **CUT** | Warp Docubot. |
| `.github/actions/bundle_arch_package/` | **KEEP** (assess) | Used by Linux Arch packaging (`script/linux/bundle_arch`). Keep if Tarp ships Arch; otherwise cut with the Arch target. |
| `.github/dependabot.yml` | **SLIM** | Keep `cargo` (security-only: `open-pull-requests-limit: 0`) + `github-actions` groups, but **drop `assignees: warpdotdev/tech-leads`** and the `github-private` registry (`dependabot.yml:11-13` and the cargo `registries:` block) — those resolve to Warp org. Aligns with the "security fast-track" sync cadence in `08-upstream-sync.md`. |
| `.github/ISSUE_TEMPLATE/`, `PULL_REQUEST_TEMPLATE/`, `STAKEHOLDERS`, `issue-triage/`, `scripts/` | **CUT / REPLACE** | Warp-org specific; replace with minimal Tarp templates. Out of CI scope; flagged for the branding/repo-hygiene pass. |

---

## 2. `ci.yml` per-job breakdown (what the replacement keeps vs drops)

| Job (`ci.yml`) | Lines | Keep in Tarp? | Notes |
|---|---|---|---|
| `params` | 42-85 | **Simplify** | Keep `dorny/paths-filter` for "affects-rust-sources" gating; **drop** the `affects-database-schema` filter (`:67-71`, persistence migrations) and the Namespace/self-hosted mac runner selection (`:73-85`) → use GitHub-hosted runners. Also drop the `repo-sync/` PR exemption (`:50-52`). |
| `tests` | 87-444 | **Keep core, strip infra** | Keep nextest unit + integration + shell-integration + doctests. **Drop:** GCP WIF auth (`:226-239`), trunk.io uploads (every `Upload results ... to trunk.io` step, `:268-281` etc.), SSH/`gcloud` remote-server tunneling (`:213-239`, `EXCLUDE_REMOTE_SERVER_TESTS_FILTER`), and the `command-signatures-v2`/`warp_js` excludes once those crates are assessed. Keep xvfb + real-display rendering tests (`:34`, `coactions/setup-xvfb`). |
| `remote-server-tests` | 446-541 | **Cut** | Already `if: false` (`:449`). `remote_server` is a Tier-A deletion (`05-removal-map.md`). |
| `database-migration` | 543-568 | **Cut (assess)** | Diesel migration check on `crates/persistence`. `persistence` survives (local sqlite for blocks/history) but its **cloud schema shrinks**; re-add a slim migration check only if Tarp keeps migrations. Not in v1 slim CI. |
| `lints` | 570-646 | **Keep** | fmt (`./script/format --check`) + clippy (`-D warnings`) + clang-format for `crates/warpui` + `app/src` ObjC. Drop the `--exclude warp_completer`/`warp_js`/`command-signatures-v2` excludes only as those crates are assessed. |
| `general-lint` | 648-704 | **Slim** | Keep `cargo deny check licenses` (`:668-673`), `check_license_config_sync` (`:675-676`), the test-file-naming + inline-test-module checks (`:692-704`), wgslfmt (`:678-683`), PSScriptAnalyzer (`:685-687`). **Drop** "Validate repo-sync markers" (`:689-690`, `warpdotdev/repo-sync/...`). |
| `wasm-lint` | 706-743 | **Cut** | `serve-wasm`/`managed_secrets_wasm` are Tier-A deletions; wasm target goes away. |
| `check-release-compilation` | 745-801 | **Keep (oss only)** | Keep the **`oss` mac/linux/win** matrix legs running `./script/bundle --channel oss --nouniversal --check-only` (`:792-801`); **drop the wasm leg** (`:774-778`). This is the OSS-build smoke test and is cheap. |
| `ci-result` | 803-858 | **Keep, simplify** | Fan-in gate so branch protection needs one required check. Drop the `database-migration` special-casing (`:846-855`) once that job is gone. |

---

## 3. DRAFT — Slim PR CI (illustrative; do not create yet)

Design goals: mirror `script/presubmit`, GitHub-hosted runners only (no Namespace
/ self-hosted / GCP / trunk.io), one fan-in required check, OSS feature set,
triggers on `main`.

```yaml
# .github/workflows/ci.yml  (DRAFT — Tarp slim PR CI)
name: Tarp CI

on:
  pull_request:
    branches: [main]            # was: master, *_release/* (ci.yml:3-6)
    types: [opened, reopened, synchronize, ready_for_review]
  push:
    branches: [main]            # post-merge gate
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  NEXTEST_PROFILE: ci
  # OSS workspace test args; warp_js / command-signatures-v2 excludes are removed
  # once those crates are assessed/deleted (see 05-removal-map.md, 03-dependencies.md).
  WORKSPACE_TEST_ARGS: --workspace --locked
  RUSTFLAGS: -C debuginfo=line-tables-only
  WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS: 1

jobs:
  lints:
    name: fmt + clippy (${{ matrix.name }})
    timeout-minutes: 25
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: macos,   name: macOS,   runner: macos-latest }
          - { os: linux,   name: Linux,   runner: ubuntu-latest }
          - { os: windows, name: Windows, runner: windows-latest }
    runs-on: ${{ matrix.runner }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/prepare_environment    # slimmed; see §1
        with:
          target_os: ${{ matrix.os }}
          is_self_hosted: 'false'
      - name: Cargo.lock up to date
        run: cargo metadata --locked --format-version=1 > /dev/null
      - name: Format check
        shell: bash
        run: ./script/format --check
      - name: Clippy
        shell: bash
        run: cargo clippy --locked --workspace --all-targets --tests -- -D warnings

  tests:
    name: tests (${{ matrix.name }})
    timeout-minutes: 30
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: macos,   name: macOS,   runner: macos-latest }
          - { os: linux,   name: Linux,   runner: ubuntu-latest }
          - { os: windows, name: Windows, runner: windows-latest }
    runs-on: ${{ matrix.runner }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/prepare_environment
        with:
          target_os: ${{ matrix.os }}
          is_self_hosted: 'false'
          install_test_deps: 'true'
      - uses: taiki-e/install-action@v2
        with: { tool: nextest }
      - name: Install shells (UNIX)
        if: ${{ matrix.os != 'windows' }}
        uses: ConorMacBride/install-package@v1
        with: { apt: 'zsh fish', brew: 'fish bash' }
      - name: Unit tests
        run: cargo nextest run ${{ env.WORKSPACE_TEST_ARGS }} -E "not package(integration)"
      - name: Integration tests (xvfb on Linux)
        if: ${{ matrix.os != 'windows' }}
        uses: coactions/setup-xvfb@v1
        with:
          run: cargo nextest run ${{ env.WORKSPACE_TEST_ARGS }} -E "package(integration)"
      - name: Doc tests
        run: cargo test ${{ env.WORKSPACE_TEST_ARGS }} --doc

  oss-build-check:
    name: OSS release compile (${{ matrix.name }})
    timeout-minutes: 20
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: macos,   name: macOS,   runner: macos-latest }
          - { os: linux,   name: Linux,   runner: ubuntu-latest }
          - { os: windows, name: Windows, runner: windows-latest }
    runs-on: ${{ matrix.runner }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/prepare_environment
        with:
          target_os: ${{ matrix.os }}
          is_self_hosted: 'false'
      - name: Check OSS bundle compiles
        shell: bash
        run: ./script/bundle --channel oss --nouniversal --check-only   # ci.yml:792-801

  ci-result:
    name: CI result
    if: ${{ !cancelled() }}
    runs-on: ubuntu-latest
    needs: [lints, tests, oss-build-check]
    steps:
      - name: Verify all jobs passed
        run: |
          echo '${{ toJSON(needs) }}' | jq -e 'to_entries | all(.value.result == "success")' \
            || { echo "::error::a required job failed"; exit 1; }
```

Note: `ci-result` is the single required status check in branch protection (mirrors
`ci.yml:803-858`), so jobs can be added/removed without re-editing protection rules.

---

## 4. DRAFT — Tag-driven release (illustrative; do not create yet)

Design goals: trigger on `v*` tags, matrix-build mac/linux/win via the existing
`oss` bundle path, attach artifacts to a GitHub Release. Stability-first: tag-driven
(no cron), so the cadence is whatever the human's tagging cadence is — burst early,
~monthly once stable. v1 ships **unsigned** with install caveats (`07-ci-and-release.md`
§3); signing is added behind secrets later.

```yaml
# .github/workflows/release.yml  (DRAFT — Tarp tag-driven release)
name: Release

on:
  push:
    tags: ['v*']               # semver tags vX.Y.Z (drops Warp channel model)
  workflow_dispatch:
    inputs:
      tag:
        description: Tag to (re)build, e.g. v0.1.0
        required: true

permissions:
  contents: write              # needed to create the GitHub Release + upload assets

jobs:
  build:
    name: Build ${{ matrix.name }}
    timeout-minutes: 90
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: macos,   name: macOS,   runner: macos-latest,   artifact_glob: 'target/release/bundle/osx/*.dmg' }
          - { os: linux,   name: Linux,   runner: ubuntu-latest,  artifact_glob: 'target/release/bundle/linux/*' }
          - { os: windows, name: Windows, runner: windows-latest, artifact_glob: 'target/release/bundle/windows/*.exe' }
    runs-on: ${{ matrix.runner }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/prepare_environment
        with:
          target_os: ${{ matrix.os }}
          is_self_hosted: 'false'
          install_release_deps: 'true'      # installs cargo-about → THIRD_PARTY_LICENSES.txt
      - name: Bundle OSS (unsigned for v1)
        shell: bash
        run: |
          # --nouniversal keeps mac to the runner arch for v1; revisit for a
          # universal2 build once an Apple Developer cert is wired in.
          # --nosign (mac) / no signing secrets => unsigned artifacts.
          ./script/bundle --channel oss --nouniversal ${{ matrix.os == 'macos' && '--nosign' || '' }}
      - uses: actions/upload-artifact@v4
        with:
          name: tarp-${{ matrix.os }}
          path: ${{ matrix.artifact_glob }}
          if-no-files-found: error

  release:
    name: Publish GitHub Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with: { path: dist, merge-multiple: true }
      - name: Create / update GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.event.inputs.tag || github.ref_name }}
          generate_release_notes: true       # until a hand-written CHANGELOG exists
          fail_on_unmatched_files: true
          files: dist/**
```

**Bundle invocation notes (verified against the scripts):**
- The single entry point is `./script/bundle` (`script/bundle:1-33`), which
  dispatches to `script/{macos,linux,windows}/bundle*` by `uname`.
- `--channel oss` selects the Sentry-free OSS feature set on all three
  (`macos/bundle:309-316`, `linux/bundle:175-182`, `windows/bundle.ps1:109-115`).
- macOS without signing secrets needs `--nosign` (`macos/bundle:133-134`) or
  `--selfsign` (`:142`); `--nouniversal` (`:149`) avoids the second-arch build.
- Linux `bundle` produces deb/rpm/arch/appimage via `bundle_deb`/`bundle_rpm`/
  `bundle_arch`/`bundle_appimage` (`07-ci-and-release.md` §Linux); narrow to
  deb + AppImage for v1 to cut matrix time, expand later.
- Windows `bundle.ps1:188-197` runs Inno Setup → `.exe` installer.
- The exact bundle **output paths must be confirmed** per platform before wiring
  `artifact_glob` (the globs above are the documented locations, not yet verified
  by a real build) — see Open Questions.

---

## 5. master → main rename impact

Renaming the default branch forces edits in these spots (do this in the same pass
that lands the slim CI):

| Location | Current | Action |
|---|---|---|
| `repo-sync.yml:4,7` | `branches: [master]` | File is **CUT** anyway. |
| `ci.yml:4` | `branches: ['master', '*_release/*']` | Replaced by the §3 draft keyed on `main`. |
| `ci.yml:54-55` | `github.ref == 'master'` gating | Gone in §3 draft (no schema/source gating retained). |
| `ci.yml:663`, `prepare_environment/action.yml:80` | `save-if: ${{ github.ref == 'refs/heads/master' }}` | Change to `refs/heads/main` (rust-cache only caches default branch). |
| `cut_new_releases.yml:39` | "Check for master branch" | File is **CUT**. |
| `populate_build_cache.yml:6` | push to `master` | File is **CUT**. |
| `dependabot.yml` | implicit default-branch targeting | No edit; follows new default automatically. |

Because the high-churn `master` references all live in files Tarp **cuts or
replaces**, the rename is low-risk for CI: the only surviving reference to fix is
the rust-cache `save-if` in the kept `prepare_environment` action.

---

## 6. Removal / sequencing order

1. **Cut the 16 Warp-internal workflows** (repo-sync, cut/delete-release,
   populate-cache, changelog, check-approvals, sync-pr-checks, the three stale/
   cleanup PR bots, docubot, feature-flag-cleanup, the three update-*-local skill
   syncs) + `release_configurations.json` + workflows/`README.md`. Pure deletes,
   no code impact, unblocks the rename.
2. **Slim `prepare_environment`** (drop GCP/Namespace/channel-config-SSH; flip
   `save-if` to `main`) and **slim `dependabot.yml`** (drop Warp assignees +
   private registry). `.github`-owned edits, safe.
3. **Land the slim PR CI** (§3) as the new `ci.yml`; set `ci-result` as the sole
   required check in branch protection. Do the **master→main rename** in the same
   PR so the new CI is keyed on `main` from the start.
4. **Land the tag release** (§4) as `release.yml`. Unsigned v1. Validate with a
   `workflow_dispatch` dry run before the first real tag.
5. **Iterate the matrix as crates are deleted** — drop the wasm legs, the
   `warp_js`/`command-signatures-v2` excludes, the `database-migration` job, and
   the remote-server filters as the corresponding Tier-A/B removals land
   (`05-removal-map.md`). CI shrinks alongside the source.
6. **Add signing/notarization later** behind secrets (macOS cert + notary,
   Windows Authenticode), and optionally a GitHub-Releases self-updater
   (`07-ci-and-release.md` §4).

This order keeps a green pipeline at every step: deletes first, then a working
slim CI, then release, then progressive narrowing.

---

## 7. Cross-cutting integration points

- **`prepare_environment` is shared by both drafts** and by the `oss` check-compile
  job. It is the one composite action Tarp keeps; its slimming (step 2) gates both
  workflows. Keep it `.github`-owned and stable.
- **The bundle scripts are the contract** between CI and packaging. The release
  workflow is intentionally thin so the logic stays in `script/`, where Tarp can
  delete the `local/dev/preview/stable` channel branches over time without ever
  touching a tracked Rust crate.
- **`cargo about` / `THIRD_PARTY_LICENSES.txt`** flows from `install_release_deps:
  'true'` → `prepare_bundled_resources` during bundling (no extra workflow step
  needed). Ties to `04-licensing.md`.
- **`ci-result` fan-in** is the single required check; both drafts and branch
  protection depend on its job-name list staying in sync.
- **`oss` channel naming** (`warp-oss` binary, `WarpOss` app, `dev.warp.WarpOss`
  bundle id) is a **branding** seam — the release artifacts will carry Warp-OSS
  names until the `06-branding-and-rename.md` pass renames the channel to `tarp`.
  CI/release should not hard-code these names; let the bundle scripts own them.

---

## 8. Risks

- **No tracked-crate edits required.** This is the safest of the de-Warp areas:
  everything here is `.github/`-owned or `script/`-owned. **No edits to
  warpui*/warp_terminal/warp_core/editor/command/etc.** are needed for CI/release.
  The only "tracked-ish" asset touched is shell integration in
  `app/assets/bundled/bootstrap/*`, which the bundle scripts copy but the CI plan
  does not modify. → **Zero merge-conflict surface against upstream terminal-core.**
- **The `oss` channel must keep compiling as features are removed.** The
  `oss-build-check` job is the guardrail; keep it green through every Tier-A/B
  removal. If `release_bundle` (the `oss` feature root) pulls a soon-to-be-deleted
  crate transitively, that surfaces here first — good.
- **GitHub-hosted runner cost/time.** Warp used Namespace/large runners
  (`ci.yml:84,117`) and self-hosted mac. Tarp on free GitHub-hosted runners will
  be slower; integration tests + universal mac builds are the long poles. Mitigate
  with `--nouniversal` for v1 and aggressive `concurrency` cancellation (already
  in §3). Re-introduce a cache/larger runners only if cold builds become painful.
- **Unsigned artifacts** trip Gatekeeper (macOS) and SmartScreen (Windows). v1
  must ship clear install docs; signing is a deliberate later step.
- **Bundle output paths unverified.** The `artifact_glob`s in §4 are from docs, not
  a real run; a first `workflow_dispatch` will confirm/fix them.
- **`label_external_contributors` / `stale` bots dropped** means manual PR hygiene
  until Tarp re-adds generic equivalents. Acceptable at Tarp's low PR volume.

---

## 9. Items touching a tracked crate

**None.** Every change is in `.github/` or `script/` (Tarp-owned per
`08-upstream-sync.md`). The plan deliberately routes all release/feature selection
through the bundle scripts' `oss` channel rather than editing crate `Cargo.toml`
defaults or any terminal-core crate. The feature-default surgery itself
(`app/Cargo.toml`) is owned by A5/the removal-map track, not by this CI plan — CI
just builds whatever the `oss` feature set resolves to.
