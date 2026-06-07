# Tarp — Fork Hardening & Release Plan

> Working plan for turning the `ramsrib/tarp` fork of Warp into a buildable,
> publishable, community-maintainable open-source terminal — with AI, cloud,
> accounts, and other non-terminal concerns removed.
>
> Status: **draft / pre-work**. The fork is currently an unmodified copy of
> Warp at upstream commit `2bb3a04b`. Nothing below has been started yet.
> See [`tarp.md`](../tarp.md) for naming & positioning.
>
> **Detailed audit evidence lives in [`docs/`](docs/README.md)** (build, crate map,
> dependencies, licensing, removal map, branding, CI/release). This file is the
> sequenced plan; `docs/` is the proof behind it.
>
> ✅ **Build verified (2026-06-05):** the fork compiles end-to-end from a clean
> checkout with **no code changes** and **no Warp-private access** — produced a
> 721 MB `warp-oss` arm64 binary (0 errors). The only setup hurdle was the macOS
> Metal Toolchain (a standard Xcode component), not anything Warp-specific. See
> [`docs/01-build-and-toolchain.md`](docs/01-build-and-toolchain.md).

---

## 1. Goal

Ship `Tarp.app` / `tarp` CLI as a plain terminal: no AI agent, no cloud, no
account system, no built-in code editor. It must:

1. Build cleanly from source on a fresh machine without Warp's private infra.
2. Carry correct, auditable licensing for everything redistributed.
3. Publish installable artifacts (macOS / Linux / Windows) via GitHub Releases.
4. Accept outside contributions like a normal OSS project.

---

## 2. Current-state snapshot (audit findings)

What the fork looks like today, so the plan is grounded in reality.

| Area | Finding |
|---|---|
| **Origin** | `origin → https://github.com/ramsrib/tarp.git`, default branch `master`. Identical to upstream Warp. |
| **Scale** | 71 crates under `crates/` + `app/`. Rust toolchain pinned to `1.92.0`. |
| **License model** | Dual: `warpui` + `warpui_core` are MIT; everything else AGPL-3.0-only. `LICENSE-MIT` + `LICENSE-AGPL` at root. Copyright "Denver Technologies, Inc." |
| **License tooling** | `about.toml` (cargo-about) + `deny.toml` (cargo-deny) drive third-party attribution; `script/check_license_config_sync` + CI enforce they stay in sync. |
| **"Private" build deps** | 5 git deps that `about.toml` *claims* aren't open-sourced yet. **Verified false (2026-06-05): all 5 are public, cloneable, and licensed.** They do **not** block the build. Only 2 of the 5 are tied to features we're removing. See §2a. |
| **Forked-OSS deps** | ~12 git deps are warpdotdev/servo forks of OSS crates (`vte`, `winit`, `font-kit`, `objc`, `pathfinder`, `yaml-rust`, `tink-rust`, `jemallocator`, `notify`, `mermaid-to-svg`, `core-foundation-rs`). Public; build fine but pin us to warpdotdev. |
| **Removal targets present** | `ai` (80 files), `editor` (82 — see ⚠ below), `onboarding` (36), `computer_use` (27), `cloud_object_models` (23), `cloud_objects` (12), `mcp` (10), `warp_server_client` (10), `warp_server_auth` (9), `cloud_object_persistence` (6), `voice_input`, `firebase`, `cloud_object_client` (1 each). |
| **Branding** | ~185 `.rs` files reference `warp.dev`/`warpdotdev`. Bundle IDs `dev.warp.*` in `app/Cargo.toml` (5 channels incl. an existing `WarpOss`). `.desktop` files, icons, plists. |
| **Telemetry** | ~99 files reference sentry/analytics/telemetry/segment. |
| **CI** | 20+ workflows in `.github/workflows/` — most are Warp-internal (Oz agents, release cutting, changelog, stale-PR bots, repo-sync). `ci.yml` (37 KB) and `create_release.yml` (83 KB) are large and Warp-specific. |
| **Packaging** | Real, mature packaging already exists: `script/{macos,linux,windows}/bundle*` (deb, rpm, AppImage, Arch, Inno installer). This is an asset to reuse. |

> ⚠ **`editor` is not the "code editor."** The `editor` crate is the
> command-line **input** buffer (content/selection/multiline/render). It is core
> to the terminal and must **stay**. Warp's removable "code editor" feature
> (Warp Code / file editing) is wired through the app + AI surfaces, not this
> crate. Confirm exact boundaries before deleting anything named "editor."

---

## 2a. The 5 git deps `about.toml` calls "not yet open-sourced"

**Investigated 2026-06-05. The `about.toml` comment is stale.** All five repos
return HTTP 200, are `git ls-remote`-cloneable anonymously, and carry explicit
licenses. **They are not build blockers.** Critically, only **2 of the 5** relate
to features we're removing — the other 3 are legitimate terminal features to
**keep**.

| Dep (repo) | License | What it does | Wired into (`.rs` files) | Verdict |
|---|---|---|---|---|
| **command-corrections** | MIT | `thefuck`-style "did you mean" — failed command + exit code + shell → suggested fix | `warp_core`, `warp_terminal`, `warp_features`, `app`, `integration` (13) | **Keep** — pure terminal UX |
| **warp-command-signatures** (command-signatures) | MIT | Command spec/signatures powering completions (args, templates, icons; e.g. `signature_by_name("kubectl")`); Fig-spec-like | `warp_completer` (17), `app` (1) | **Keep** — core to completions |
| **warp-workflows** (workflows) | Apache-2.0 | "Workflows" = saved/parameterized command snippets (community workflows repo) | `cloud_object_models`, `app`, `integration` (5) | **Keep**, but currently entangled with the cloud layer (see note) |
| **session-sharing-protocol** | AGPL-3.0 | Real-time session sharing / collaboration protocol types (Guest, Role, ACL, sharer/viewer, ParticipantList) | **`app` (66)**, `cloud_object_*` (4), `warp_terminal` (2) | **Remove** — cloud/collab |
| **warp_multi_agent_api** (warp-proto-apis) | AGPL-3.0 | Protobuf API for AI multi-agent conversations (Message, ConversationData, response_event, StreamInit, file-diff results) | **`app` (56)**, `ai` (11), `persistence` (2), `integration` (1) | **Remove** — AI backbone |

**Consequences for the plan:**

1. **No build blocker.** A green baseline (M1) is achievable today with all 5 intact.
2. **Don't strip 3 of them.** command-corrections, command-signatures, and
   workflows are exactly the "good terminal" features Tarp wants. They are
   permissively licensed (MIT/MIT/Apache).
3. **The 2 removals are `app/`-layer surgery, not crate deletions.**
   `session-sharing-protocol` touches **66 files in `app/`** + 2 in
   `warp_terminal`; `warp_multi_agent_api` touches **56 in `app/`**. The effort
   is unpicking these from the app, not deleting a `Cargo.toml` line.
4. **`warp-workflows` is referenced from `cloud_object_models`** — a "keep"
   feature currently entangled with the cloud layer being deleted. Untangling
   workflows from cloud sync is a WS2 sub-task.
5. **`about.toml`/`deny.toml` must be corrected** (WS5): these crates now have
   explicit licenses, so they should be *included* in attribution generation,
   not skipped via the ignore list — and the `--fail` flag the comment mentions
   can be re-added.

---

## 3. Workstreams

Ordered roughly by dependency. WS1 (baseline build) should happen before any
deletion so we have a known-good reference and can bisect breakage.

### WS0 — Repo & governance setup
- [x] **Name decided: "Tarp"** (kept). Published rationale is the tarp metaphor. Trademark checklist in [`docs/04-licensing.md`](docs/04-licensing.md#trademark-separate-from-the-code-license).
- [ ] Ship a **distinct name, logo, and visual identity**; remove all upstream brand assets (verify the built artifact).
- [ ] Add **affiliation disclaimer** to README + About: "Tarp is an independent fork, not affiliated with / endorsed by the upstream project."
- [ ] Run a **trademark clearance search** before first public release.
- [ ] Rename default branch `master` → `main` (GitHub setting + `git remote set-head`, update any branch refs in workflows/scripts).
- [ ] Decide governance model: solo-maintained vs. open contributions; write it down.
- [ ] Replace `CODE_OF_CONDUCT.md` contact (`warp-coc@warp.dev`), `SECURITY.md`, `CONTRIBUTING.md` with Tarp equivalents.
- [ ] Rewrite `README.md` for Tarp (strip Oz/build.warp.dev/careers/Slack-community sections; keep build instructions).
- [ ] Add `NOTICE`/attribution stating Tarp is a fork of Warp, as required for AGPL compliance (keep upstream copyright notices).
- [ ] Prune `.github/workflows/` to the handful we actually run (see WS4). Remove Oz/Docubot/repo-sync/stale-bot/changelog automation.
- [ ] Update `ISSUE_TEMPLATE`, `PULL_REQUEST_TEMPLATE`, `STAKEHOLDERS`, `dependabot.yml`.
- [ ] Decide fate of `WARP.md`, `FAQ.md`, `specs/`, `.warp/`, `skills-lock.json`, `.agents/`, `.mcp.json` (Warp-dev tooling — likely remove or replace).

### WS1 — Establish a clean baseline build (no code changes)
- [ ] Run `./script/bootstrap` on a clean macOS machine; capture every prerequisite.
- [ ] `cargo build` (default-members) and full `cargo run`; record what breaks.
- [x] ~~Determine whether the 5 "proprietary" git deps are publicly cloneable.~~ **Done (2026-06-05): all public & licensed; not blockers. See §2a.**
- [ ] Try `script/macos/bundle` to produce a `.app`; confirm it launches.
- [ ] Document the working build in a `BUILD.md` (toolchain, system deps, env vars like `SERVER_ROOT_URL`).
- [ ] Establish whether `with_local_server` / server features can be left off entirely for a pure-terminal build.
- [ ] Record build time / artifact size as a baseline to compare against post-strip.

### WS2 — Strip non-terminal features ("de-Warp")
Approach: prefer **feature-gating then deleting** crate-by-crate, rebuilding after
each removal, rather than one giant deletion. Use the workspace graph to find
reverse-dependencies before deleting a crate.

> **Maintainability constraint (from [`docs/08-upstream-sync.md`](docs/08-upstream-sync.md)):**
> do removals **in the `app/` layer + via feature flags**, and keep the
> **tracked-from-upstream** terminal-core crates (`warpui*`, `warp_terminal`,
> `warp_core`, `editor`, `command`, `warp_completer`, `vim`, `syntax_tree`, …) as
> structurally close to upstream as possible. Every gratuitous edit to a tracked
> crate is a future merge conflict. Define the tracked-vs-owned path split before
> cutting.
- [x] Wave 0 analysis fan-out → file-level specs in [`docs/removal/`](docs/removal/README.md); product decisions locked (D1–D4); reconciliations applied. Worklog: [`docs/PROGRESS.md`](docs/PROGRESS.md).
- [x] **Step 1 done (branch `dewarp`):** minimal `default` 187→49 + `gui=[]`; builds + bundles + launches. The "shrink default first" keystone.
- [ ] Map the dependency graph (`cargo tree`/`cargo-depgraph`) for each removal-target crate to find what references it.
- [ ] **AI**: remove `ai`, `computer_use`, `voice_input`, MCP (`mcp`), and `warp_multi_agent_api` (56 `app/` files + `ai`/`persistence`); strip `app/src/settings/ai.rs` and AI UI surfaces.
- [ ] **Cloud**: remove `cloud_objects`, `cloud_object_*`, `firebase`, `warp_server_client`, `warp_server_auth`, `session-sharing-protocol` (66 `app/` files + `warp_terminal`), `warp_web_event_bus` (assess).
- [ ] **Keep, don't strip** (§2a): `command-corrections`, `warp-command-signatures`, `warp-workflows` — terminal features, permissively licensed.
- [ ] **Untangle `warp-workflows` from the cloud layer**: it's currently pulled via `cloud_object_models`; rewire so workflows survive cloud removal.
- [ ] **Accounts/auth**: remove login/onboarding (`onboarding`), auth flows; ensure first-run works with zero account.
- [ ] **Code editor feature**: identify and remove the file-editing surfaces (NOT the input `editor` crate — see ⚠).
- [ ] **Telemetry**: remove sentry/analytics/segment wiring (~99 files); strip `script/sentry_*`.
- [ ] Remove only the 2 removal-target git deps (`session-sharing-protocol`, `warp_multi_agent_api`) from `Cargo.toml` once consumers are gone; confirm `Cargo.lock` resolves. Keep the other 3.
- [ ] Decide on the warpdotdev **forked-OSS** deps: keep pinned, re-point to upstream, or vendor under a Tarp org. (Lowest-risk: keep for now, revisit.)
- [ ] Delete dead `crates/`, `specs/`, assets; run clippy/tests after each major cut.

### WS3 — Rebrand to Tarp
- [ ] Bundle identifiers `dev.warp.*` → chosen namespace (e.g. `dev.tarp.Tarp`) in `app/Cargo.toml`, plists, `.desktop`, DockTilePlugin.
- [ ] App/CLI names: `Warp.app` → `Tarp.app`, binary `warp` → `tarp` (assess CLI-name coupling, shell-integration scripts in `app/assets/bundled/`).
- [ ] Icons/logo (`script/compile_icon`, `images/`, `resources/`) — the "literal tarp" visual from `tarp.md`.
- [ ] Sweep user-visible "Warp" strings (settings, about, window title); keep upstream attribution where required.
- [ ] `about.hbs` / `about.toml` product name; `channel_versions` + channels (`stable/preview/dev/local/oss`) — collapse to the channels we'll actually ship.
- [ ] Update `flake.nix`, `docker/`, `Dockerfile`-adjacent references.

### WS4 — Release engineering
- [x] Lean GitHub Actions release workflow (tag `v*` → preflight gate → macOS build → publish Release). `release.yml`. Linux/Windows legs still TODO.
- [x] Reuse `script/macos/bundle` for the `.app`/dmg (oss channel). Linux/Windows bundle scripts exist; not yet wired into CI.
- [x] **Code signing + notarization (macOS):** Developer ID + notarized (App Store Connect API key); `v0.1.0` verified `spctl` accepted. Windows Authenticode still TODO.
- [x] Versioning: semver tags `vX.Y.Z` (Warp channel/version model dropped).
- [ ] Set up an auto-update story or explicitly document "no auto-update" (currently none; `autoupdate_config: None`).
- [x] Minimal CI: fmt + Linux build on PR (`ci.yml`); preflight gate on release.
- [x] Install docs in README + `RELEASING.md` (per-platform `INSTALL.md` not needed yet — macOS only).

### WS5 — License & legal audit (full)
- [ ] Confirm AGPL-3.0 + MIT split is preserved and we comply (AGPL network-use clause is moot once cloud is gone, but keep the license).
- [ ] Keep upstream copyright notices; add Tarp copyright alongside, don't replace.
- [ ] Re-run `cargo about` / `cargo deny` after dep changes; regenerate third-party attributions (`docs.warp.dev/help/licenses` equivalent → ship a `THIRD_PARTY_LICENSES`).
- [ ] Fix stale `about.toml` comment: the 5 git deps **are** open-sourced (MIT ×2, Apache-2.0, AGPL ×2). Remove the kept 3 from the cargo-about/deny ignore lists so they're attributed; the 2 removed deps drop out naturally. Re-add `--fail` to the generate invocation.
- [ ] Verify `about.toml`/`deny.toml` stay in sync (their check script) after the dep changes.
- [ ] Trademark check: "Warp" name/logo are Warp's — ensure full removal from shipped artifacts. Tarp branding must be clearly distinct.
- [ ] Audit bundled assets (fonts via `patch_font_with_warp_glyph`, icons, sounds) for redistribution rights.
- [ ] Confirm the forked-OSS git deps carry compatible licenses (they're in `about.toml`'s accepted list).

### WS6 — Community & maintenance
- [ ] `CONTRIBUTING.md` rewrite: how to build, PR flow, no Oz/agent bot expectations.
- [ ] Issue/PR templates pointed at Tarp.
- [ ] Decide support channels (drop Warp Slack; GitHub Discussions?).
- [ ] Roadmap / "non-goals" doc (loudly: no AI, no cloud, no accounts) — reuse `tarp.md` positioning.

---

## 4. Risk register

| Risk | Impact | Mitigation |
|---|---|---|
| ~~Proprietary deps not cloneable~~ | ~~Build blocked~~ | **Resolved (§2a): all 5 public & licensed. Not a blocker.** |
| `session-sharing-protocol` (66) + `warp_multi_agent_api` (56) are woven deep into `app/` | Removal is large, error-prone surgery | Scope reverse-deps in `app/` before cutting; gate-then-delete; rebuild per step. The 2 removals are the bulk of WS2. |
| Removing AI/cloud cascades into core terminal crates | Large breakage | Map reverse-deps before deleting; gate-then-delete; rebuild per step. |
| Deleting the `editor` crate by mistake | Breaks input | Treat `editor` as core; only remove the file-editing *feature*. |
| Auto-updater / release infra deeply tied to Warp cloud | Can't ship updates | WS4 replaces with plain GitHub Releases; document "manual update" v1. |
| macOS notarization / Windows signing certs | Users get scary warnings | ✅ macOS signed (Developer ID) + notarized as of v0.1.0. Windows Authenticode still TODO. |
| Forked-OSS deps pinned to warpdotdev disappear | Build breaks long-term | Vendor or re-point to upstream in WS2 (deferred, low urgency). |
| Trademark/brand residue in shipped binary | Legal | WS3 + WS5 sweeps; verify the built artifact, not just source. |
| Trademark exposure | Legal (trademark) | Distinct name/logo/visual identity; full removal of upstream brand assets; affiliation disclaimer; nominative fair use only; clearance search. Checklist in [`docs/04-licensing.md`](docs/04-licensing.md#trademark-separate-from-the-code-license). Lock brand identity before first public release. |

---

## 5. Open questions (need your call)

1. **Branch rename now or after first green build?** (Low risk either way; I'd do it first.)
2. **Platform priority** — mac-first, or all three from day one?
3. **Forked-OSS warpdotdev deps** — keep pinned for v1, or vendor/re-point immediately?
4. **Channels** — collapse Warp's 5 channels to a single stable build, or keep stable+preview?
5. **Updater** — remove entirely (manual download), or build a minimal self-update?
6. **History** — keep full upstream git history (good for AGPL/attribution + rebasing upstream fixes), or squash? (Recommend keep.)
7. **Upstream sync** — analyzed in [`docs/08-upstream-sync.md`](docs/08-upstream-sync.md). Recommendation: selective **cherry-pick / path-scoped** sync of terminal-core fixes only (never full merge — upstream runs ~14 commits/day, mostly AI/cloud), with a tracked-vs-owned **path split** that governs how M2 is done. Confirm cadence (monthly + security fast-track) and that you're OK depending on upstream staying AGPL-published.

---

## 6. Suggested sequencing (milestones)

> **Strategy pivot (2026-06-05, [ADR-005](docs/DECISIONS.md)):** v1 reaches the
> no-AI/no-cloud terminal by **disabling** those surfaces (minimal default feature
> set), not by deleting their source. Full AI+cloud source deletion is deferred to
> an optional later project (it's a ~600–700-file coordinated surgery with high
> upstream divergence — see [`docs/removal/ai-removal-feasibility.md`](docs/removal/ai-removal-feasibility.md)).
> So M2 is reinterpreted as "features disabled," and the path to release is M3→M4.

- **M0 — Hygiene**: WS0 branch rename + README/license/CoC swap. (small, fast)
- **M1 — Green baseline**: ✅ **done** — unmodified build verified, `WarpOss.app` bundles + launches, documented in [`BUILD.md`](BUILD.md).
- **M2 — De-Warped build (disable)**: ✅ **substantially done** — minimal default feature set (187→49) disables AI/cloud/editor; `voice_input` deleted; builds + launches. Full source deletion deferred (ADR-005).
- **M3 — Tarp brand**: WS3 rename + icons; artifact says "Tarp" everywhere. ← **next**
- **M4 — First release**: WS4 + WS5 GitHub Release with installable artifacts + clean license attribution.
- **M5 — Open for contributors**: WS6 docs, templates, CI.
- **M6 (optional, later)** — full AI+cloud **source deletion** per the `docs/removal/` specs, if the binary-size / source-purity win justifies the effort + divergence.

---

*Plan doc; the code work lives on branch `dewarp`. See [`docs/PROGRESS.md`](docs/PROGRESS.md) for the work log and [`docs/DECISIONS.md`](docs/DECISIONS.md) for rationale.*
