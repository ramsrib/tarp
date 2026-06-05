# A3 — Telemetry / Analytics / Crash-Reporting Removal Spec

Status: analysis only (2026-06-05). No source modified. This is the file-level
worklist for stripping the telemetry, analytics, and crash-reporting (Sentry /
minidumper / crash-handler) surface from the fork.

Cross-references: `docs/05-removal-map.md` (telemetry = 315 `app/src` files,
"stub a facade to no-op first"), `docs/08-upstream-sync.md` (tracked-vs-owned
path split). This spec refines those into concrete edits.

---

## 0. TL;DR — the single most important finding

**The OSS channel already ships telemetry- and crash-reporting-free**, and the
mechanism is clean and already in-tree:

- `crates/warp_core/src/channel/state.rs:38-55` — `ChannelState::init()` defaults
  to `Channel::Oss` with `telemetry_config: None` and `crash_reporting_config: None`.
- Every accessor degrades to an empty/no-op when those are `None`:
  - `rudderstack_non_ugc_destination()` / `rudderstack_ugc_destination()`
    (`state.rs:296-319`) → `RudderStackDestination::default()` (empty url/key).
  - `sentry_url()` (`state.rs:342-350`) → empty `Cow`.
  - `telemetry_file_name()` (`state.rs:174-182`) → empty.
  - `is_telemetry_available()` / `is_crash_reporting_available()`
    (`state.rs:188-198`) → `false` (this is the flag the UI checks to hide the
    privacy toggles).
- The bundle scripts already drop the Sentry features for the `oss` channel:
  - `script/macos/bundle:309-317` (oss branch sets `FEATURES="release_bundle,extern_plist"`, dropping `cocoa_sentry`).
  - `script/linux/bundle:179-181` (oss branch sets `FEATURES="release_bundle"`, dropping `crash_reporting`).
  - `script/windows/bundle.ps1:113-115` (same).
- The OSS bundle metadata block in `app/Cargo.toml` (`[package.metadata.bundle.bin.oss]`, ~lines 998-1006) has **no `osx_frameworks` entry** — i.e. it never embeds `Sentry.framework` (unlike the `stable`/`preview`/`dev` blocks at lines 1012-1036).

**Implication:** A buildable, fully telemetry-free Tarp is reachable *purely by
feature-flag selection* — `cargo build -p warp --features release_bundle`
(no `crash_reporting` / `cocoa_sentry`, default channel is `Oss`). No tracked-crate
edits required for a working no-op. The full removal below is about deleting the
now-dead code and the build/bundle/script machinery, not about making it "off".

The two-phase strategy the removal-map calls for is therefore:
1. **Phase 1 (stub to no-op):** already achieved by the OSS channel +
   not enabling `crash_reporting`/`cocoa_sentry`. Verify, lock it in, then
2. **Phase 2 (delete the dead code):** remove the `app/`-owned telemetry/crash
   subsystems and the bundled Sentry artifacts, and (optionally, accepting a
   tracked-crate edit) excise the now-`#[cfg(feature="crash_reporting")]`-gated
   sentry code from tracked crates.

---

## 1. Scope

In scope:
- App telemetry pipeline (RudderStack event collection + send): `app/src/server/telemetry/**`.
- The telemetry call-site macros and the `TelemetryEvent` trait/registry.
- Crash reporting / Sentry: `app/src/crash_reporting/**`, the macOS objc shim,
  the Linux/Windows minidumper path, and Sentry init in `app/src/lib.rs`.
- Bundled Sentry **xcframework** + the `script/sentry_*` and
  `script/macos/update_sentry_cocoa` machinery + `app/build.rs` link logic.
- Cargo features: `crash_reporting`, `cocoa_sentry`, `send_telemetry_to_file`,
  `with_sandbox_telemetry`, `record_app_active_events`,
  `global_ai_analytics_collection`, `global_ai_analytics_banner`,
  `log_expensive_frames_in_sentry`, `heap_usage_tracking` (depends on crash_reporting).
- Sentry-related deps: `sentry`, `sentry-log`, `minidumper`, `crash-handler`.

Out of scope (handled by other A-tasks, only noted where they intersect):
- AI/agent telemetry files under `app/src/ai/**/telemetry*.rs` (go with the AI
  removal — A?-ai); listed here only so they're not double-counted.
- `crates/onboarding/src/telemetry*.rs` (goes with onboarding removal).
- `crates/ai/src/telemetry.rs` (goes with `ai` crate deletion).

Honors project rules:
- The **`editor` crate is untouched** (it is the input buffer, not in scope).
- Tracked-from-upstream crates (`warp_core`, `warpui_core`, `warp_logging`) carry
  some telemetry/sentry code. Edits there are flagged in §6 as merge-conflict risk;
  Phase 1 avoids them entirely by leaning on feature gating.

---

## 2. The facade / abstraction map (what call sites go through)

There are **two stacked facades**. Understanding both is what lets us cut at the
smallest, most-isolating point.

### 2a. Analytics (RudderStack) — emission path

```
~800 call sites
  send_telemetry_from_ctx! / send_telemetry_from_app_ctx!     [DEFINED IN TRACKED warp_core]
      crates/warp_core/src/telemetry.rs:147 and :176
        └─ event.enablement_state().is_enabled()  (gates the whole thing)
        └─ warpui_core::record_telemetry_from_ctx! / record_telemetry_on_executor!  [TRACKED warpui_core]
              crates/warpui_core/src/telemetry/mod.rs:17 and :36
                └─ warpui_core::telemetry::record_event()  [TRACKED]
                      crates/warpui_core/src/telemetry/mod.rs:75 → global EventStore (in-memory queue)
                          crates/warpui_core/src/telemetry/event_store.rs

  send_telemetry_sync_* / send_telemetry_on_executor!         [DEFINED IN APP — owned]
      app/src/server/telemetry/macros.rs:5,40,75
        └─ ServerApi::send_telemetry_event(...) → TelemetryApi  [APP — owned]
```

Drain + network egress (all APP-owned):
```
TelemetryCollector (singleton, periodic flush + shutdown flush)
    app/src/server/telemetry/collector.rs
      └─ ServerApi::flush_telemetry_events → TelemetryApi::flush_events
            app/src/server/telemetry/mod.rs:94
              └─ warpui::telemetry::flush_events() drains the queue
              └─ send_batch_messages_to_rudder() → POST to RudderStack
                    app/src/server/telemetry/mod.rs:303,380
                      └─ ChannelState::rudderstack_*_destination()  (empty in OSS)
```

Call-site macro frequency (counted with `rg -o ... | uniq -c`):
- `send_telemetry_from_ctx!` — **774** (warp_core macro; the dominant path)
- `send_telemetry_sync_from_app_ctx!` — 14 (app macro)
- `send_telemetry_sync_from_ctx!` — 9 (app macro)
- `send_telemetry_on_executor!` — 7 (app macro)
- bare `send_telemetry_event` — 6
- `record_telemetry_on_executor!` — 2, `record_telemetry_from_ctx!` — 1

**The smallest, most-isolating cut** for analytics is the network egress, not the
800 call sites: `TelemetryApi` (`app/src/server/telemetry/mod.rs`) and
`TelemetryCollector` (`collector.rs`). With those neutered, every call site still
compiles (the macros just enqueue into an in-memory queue that is never drained
to the network). And in the OSS channel the destinations are already empty, so the
POSTs are no-ops even before any edit. The 800 call sites can then be removed
lazily / left in place behind the always-disabled path without breaking the build.

### 2b. Crash reporting (Sentry) — error/crash path

```
report_error! / report_if_error! macros + Error::report()     [TRACKED warp_core]
    crates/warp_core/src/errors.rs (macros + impls)
      └─ #[cfg(feature="crash_reporting")] sentry::capture_error()     errors.rs:123-124
      └─ #[cfg(feature="crash_reporting")] sentry::integrations::anyhow::capture_anyhow()  errors/anyhow.rs:28-29
    crates/warp_logging/src/native.rs — sentry-log layer, all behind crash_reporting feature
    crates/warp_logging/Cargo.toml:36 — crash_reporting = ["dep:sentry","dep:sentry-log","warp_core/crash_reporting"]
    crates/warp_core/Cargo.toml:9     — crash_reporting = ["dep:sentry","dep:sentry-log"]
    crates/ai/Cargo.toml:12           — crash_reporting = ["dep:sentry"]

App-side Sentry subsystem (APP-owned, all behind feature="crash_reporting"/"cocoa_sentry"):
    app/src/crash_reporting/mod.rs   (617 LOC) — init/uninit, tags, scrubbing, set_user_id, crash()
    app/src/crash_reporting/mac.rs   (107)     — cocoa_sentry objc bridge
    app/src/crash_reporting/linux.rs (26)
    app/src/crash_reporting/sentry_minidump.rs (442) — minidumper + crash-handler server
    app/src/platform/mac/objc/crash_reporting.{h,m}  — objc Sentry SDK shim
```

**The crash-reporting facade is the cargo `crash_reporting` feature itself.**
Every sentry reference in tracked crates is `#[cfg(feature = "crash_reporting")]`.
Not enabling the feature = zero sentry, zero edits to tracked crates. This is the
ideal Phase-1 cut and it is already how OSS bundles work.

---

## 3. Cargo feature flags (the control surface)

In `app/Cargo.toml`:

| Feature | Line | Pulls / does | Action |
|---|---|---|---|
| `crash_reporting` | 477-484 | `dep:sentry`, `dep:sentry-log`, `dep:minidumper`, `dep:crash-handler`, `warp_logging/crash_reporting`, `ai/crash_reporting` | **Never enable; then delete the feature + the deps** |
| `cocoa_sentry` | 469 | `= ["crash_reporting"]` (macOS objc Sentry path) | Delete |
| `log_expensive_frames_in_sentry` | 769 | `= []` (used in perf logging to sentry) | Delete + remove guarded call sites |
| `heap_usage_tracking` | 737 | `= ["jemalloc_pprof", "crash_reporting"]` | Drop the `crash_reporting` dep (or delete feature) |
| `send_telemetry_to_file` | 886 | `= []` (writes events to a local log file) | Delete + remove `FeatureFlag::SendTelemetryToFile` branches |
| `record_app_active_events` | 795 | `= []` (active-usage heartbeat events) | Delete (with `RecordAppActiveEvents` flag) |
| `global_ai_analytics_collection` | 697 | `= []` — **IN DEFAULT SET** (`app/Cargo.toml:516`) | Remove from `default`, delete |
| `global_ai_analytics_banner` | 696 | `= []` | Delete |
| `with_sandbox_telemetry` | (grep) | sandbox-only telemetry override | Delete |

Confirmed gating in `app/src/features.rs`:
- `CrashReporting` flag is `#[cfg(feature = "crash_reporting")]` (features.rs:32-33).
- `GlobalAIAnalyticsCollection` is `#[cfg(feature="global_ai_analytics_collection")]` (features.rs:128-129); `GlobalAIAnalyticsBanner` at 126.
- `RecordAppActiveEvents` (features.rs:37), `SendTelemetryToFile` (features.rs:229).

Deps to delete from `app/Cargo.toml` once the features are gone:
- `sentry` (line 146), `sentry-log` (line 177), `crash-handler` (line 376),
  `minidumper` (line 377). Also drop `sentry`/`sentry-log` from
  `crates/warp_core/Cargo.toml:38-39`, `crates/warp_logging/Cargo.toml:22-23`,
  `crates/ai/Cargo.toml:37` **if** Phase 2 edits to tracked crates are accepted
  (see §6). Then regenerate `Cargo.lock`.

> Note: `crash_reporting`/`cocoa_sentry` are **not in the `default` set** — only
> `global_ai_analytics_collection` is (`app/Cargo.toml:516`). So default `cargo
> build` already excludes Sentry; only the bundle scripts add it for non-oss
> channels.

---

## 4. File-level worklist (grouped)

### Group A — App telemetry pipeline (DELETE; app-owned)
Delete the directory `app/src/server/telemetry/` in full:
- `mod.rs` (411) — `TelemetryApi` + RudderStack send/flush/persist.
- `collector.rs` (229) — `TelemetryCollector` singleton + periodic/shutdown flush.
- `context.rs` (99), `context_provider.rs` (26) — telemetry context attach.
- `events.rs` (**7363**) + `events_tests.rs` — the giant event catalog (every
  `TelemetryEvent` impl + `register_telemetry_event!`). 20 telemetry-impl files
  reference these.
- `macros.rs` (93) — `send_telemetry_sync_*` / `send_telemetry_on_executor!`.
- `rudder_message.rs` (249) — RudderStack wire types.
- `secret_redaction.rs` (127) + tests — UGC scrubbing.
- `mod_tests.rs`, `LICENSE-RUDDER-SDK-RUST.txt`.

Then remove the module wiring:
- `app/src/server/mod.rs` — drop `mod telemetry;` / `pub mod telemetry;` and
  re-exports (find the `telemetry` module declaration).
- `app/src/server/telemetry_ext.rs` + `telemetry_ext_tests.rs` — the
  `TelemetryExt` trait (`to_rudder_batch_message`); delete and unwire from
  `app/src/server/mod.rs`.
- `app/src/lib.rs:287` — import of `TelemetryCollector` etc.; `lib.rs:1603-1605`
  (collector construction + `initialize_telemetry_collection`); `lib.rs:2148-2149`
  (`flush_telemetry_events_for_shutdown` on shutdown). Remove these.

### Group B — Telemetry call sites (~800; app-owned, mechanical)
Every `send_telemetry_from_ctx!(...)` / `send_telemetry_from_app_ctx!(...)` /
`send_telemetry_sync_*` / `send_telemetry_on_executor!` call across `app/src`.
- Counts in §2a. Spread across ~324 files referencing `telemetry`.
- **Lowest-risk approach:** keep the `warpui_core` no-op queue (Group D), and
  the macros keep compiling; remove call sites opportunistically as their owning
  feature (AI, sharing, etc.) is deleted by the sibling A-tasks. The
  `send_telemetry_sync_*` macros (app-owned, in `macros.rs`) disappear with
  Group A, so the ~30 sync call sites MUST be removed (or the macros stubbed).
- The remaining ~774 `send_telemetry_from_ctx!` calls depend on the **tracked**
  warp_core macro (§6) — leave the macro in place (no-op via empty queue) and
  delete call sites lazily to avoid tracked-crate edits.

### Group C — Crash reporting / Sentry (DELETE; app-owned)
- `app/src/crash_reporting/mod.rs` (617), `mac.rs` (107), `linux.rs` (26),
  `sentry_minidump.rs` (442) — delete the whole `app/src/crash_reporting/` dir.
- `app/src/platform/mac/objc/crash_reporting.h` + `.m` — delete (objc Sentry shim).
- `app/src/crash_recovery.rs` + `app/src/workspace/view/crash_recovery.rs` —
  ASSESS: crash *recovery* (restoring session after a crash) is separate from
  crash *reporting*. Keep recovery if it doesn't pull Sentry; it's gated by
  `enable_crash_recovery` cfg alias (`app/build.rs:23`, linux_or_windows) not by
  `crash_reporting`. **Likely KEEP.**
- `app/src/lib.rs` — remove all `#[cfg(feature="crash_reporting")]` blocks:
  - `lib.rs:28-29` (`mod crash_reporting;`)
  - `lib.rs:500-501` `needs_crash_reporting()` + `lib.rs:777-783` `sentry::Hub::main()`
  - `lib.rs:640-644` minidump-server launch branch (`run_minidump_server`)
  - `lib.rs:842-843,851,874,897,1057,1077,1080` the `pre_sentry_errors` buffer
    plumbing (the whole "buffer errors until Sentry is ready" machinery)
  - `lib.rs:1031-1032` `set_client_type_tag`
  - `lib.rs:1338-1347` `crash_reporting::init(ctx)` + replaying pre-init errors
    (`capture_anyhow`)
  - `lib.rs:1473` `is_crash_reporting_enabled` propagation
  - `lib.rs:2187-2188` `crash_reporting::uninit_sentry()` on shutdown
- `app/src/workspace/mod.rs` — the "Crash the app (for testing Sentry)" debug
  command/menu item; remove.

### Group D — `warpui_core` in-memory telemetry queue (TRACKED — leave; see §6)
- `crates/warpui_core/src/telemetry/{mod.rs,event_store.rs,event_store_tests.rs}`
  — the global `EventStore` + `record_event`/`flush_events`/`create_event`.
  **Leave in place** as a tracked-upstream no-op sink. With Group A gone, nothing
  drains the queue (`flush_events` only called by the deleted `TelemetryApi`), so
  it becomes a bounded in-memory buffer. Optional Phase-2 hardening: cap or
  short-circuit `record_event` — but that's a tracked-crate edit (§6).
- `crates/warpui_core/src/{app_focus_telemetry.rs,app_focus_telemetry_tests.rs}`
  (tracked) — emits focus telemetry events. Leave; it routes through the same
  no-op queue. Flag as tracked-edit risk only if we want it fully gone.

### Group E — Settings & privacy UI (app-owned)
- `app/src/settings/privacy.rs` — `WarpDrivePrivacySettings` group with
  `IsTelemetryEnabled`, `IsCrashReportingEnabled`, `CustomSecretRegexList`.
  ASSESS: these toggles are already hidden when
  `ChannelState::is_telemetry_available()` / `is_crash_reporting_available()` are
  false (OSS). Can keep the settings (harmless, no effect) or delete the group.
  Deleting requires unwiring `app/src/settings/init.rs` (`WarpDrivePrivacySettings::register`)
  and `app/src/settings/initializer.rs`.
- `app/src/settings_view/privacy_page.rs` — privacy settings UI page. The
  telemetry/crash sections are guarded by `if !ChannelState::is_telemetry_available()` /
  `is_crash_reporting_available()` returns (already hidden in OSS). Delete the
  sections or the whole page section.
- `app/src/settings_view/telemetry.rs` — settings-view telemetry events; delete.
- `app/src/auth/auth_view_shared_helpers.rs`, `auth_view_body.rs`,
  `app/src/workspace/view.rs`, `app/src/terminal/view.rs` — the
  `GlobalAIAnalyticsBanner` / `is_telemetry_available()` banner conditionals;
  remove the banner branches when `global_ai_analytics_*` features are deleted.
- `app/src/settings/ai.rs` — references `PrivacySettings` for AI-UGC telemetry;
  handled with the AI removal but note the `should_collect_ai_ugc_telemetry`
  coupling (`app/src/server/telemetry/mod.rs:325`).

### Group F — Scattered app telemetry modules (app-owned; mostly go with their feature)
These are feature-cluster telemetry, removed alongside the owning feature by
sibling tasks — listed for completeness so they aren't missed:
- `app/src/antivirus/telemetry.rs`, `app/src/notebooks/telemetry.rs`,
  `app/src/tab_configs/telemetry.rs`, `app/src/workspace/view/vertical_tabs/telemetry.rs`,
  `app/src/code/lsp_telemetry.rs`, `app/src/code_review/telemetry_event.rs`,
  and all `app/src/ai/**/telemetry*.rs` (AI task).

### Group G — Bundled Sentry artifacts + build/scripts (DELETE; app-owned)
- **xcframework (not git-tracked as files):** downloaded at build time by
  `script/macos/update_sentry_cocoa` into `app/frameworks/{default,dev}/Sentry-Dynamic-WithARM64e.xcframework/...`.
  Delete the script and any committed `app/frameworks/**/Sentry*` dirs if present.
- `app/build.rs`:
  - `build.rs:38` call to `build_and_link_sentry()` (remove).
  - `build.rs:240-281` `build_and_link_sentry()` (delete fn).
  - `build.rs:283-315` `download_sentry_framework()` (delete fn).
  - `build.rs:317-328` `compile_sentry_objc_lib()` (delete fn).
- `script/sentry_create_release.sh`, `script/sentry_upload_dif.sh`,
  `script/macos/update_sentry_cocoa` — delete. Grep CI for callers
  (`.github/`) and remove the upload-dif / create-release steps.
- `script/macos/run:` — the `SENTRY_FRAMEWORK=...` copy-into-bundle block and the
  `if [[ ",$FEATURES," =~ ",cocoa_sentry," ]]` branch; remove.
- `script/macos/bundle:102` (`FEATURES="release_bundle,cocoa_sentry,extern_plist"`),
  `script/linux/bundle:24`, `script/windows/bundle.ps1:17` — drop the sentry feature
  from the default FEATURES; collapse the now-redundant oss-channel special-cases
  (`script/macos/bundle:309-317`, `script/linux/bundle:179-181`, `script/windows/bundle.ps1:113-115`).
- `app/Cargo.toml:1012-1014, 1023-1025, 1034-1036` — remove the `osx_frameworks =
  [ ".../Sentry.framework" ]` entries from the stable/preview/dev bundle metadata
  blocks (the oss block already has none).
- `app/Cargo.toml` package `include`/`exclude` (lines ~1013, ~1024, ~1035 hit by
  grep) — drop the framework paths.
- `specs/daemon-sentry-initialization` — spec doc; delete or mark obsolete.

### Group H — Channel config simplification (TRACKED warp_core — optional, §6)
After full removal, the `Option<TelemetryConfig>` / `Option<CrashReportingConfig>`
in `crates/warp_core/src/channel/config.rs:21,25` are always `None`. Leaving them
is the **low-divergence** choice (keeps tracked crate upstream-shaped). Removing
the fields + `TelemetryConfig`/`RudderStackConfig`/`CrashReportingConfig` structs
(config.rs:88-137) and the accessors (state.rs:174-198,296-350) is a tracked-crate
edit — defer unless a clean break is wanted.

---

## 5. Removal / sequencing order (de-risked)

1. **Verify the no-op baseline (no edits).** Build OSS:
   `cargo build -p warp --features release_bundle` (default channel Oss, no
   `crash_reporting`/`cocoa_sentry`). Confirm it builds and runs and emits nothing.
   This *is* Phase-1 stub — telemetry/crash are already inert.
2. **Cut the network egress (Group A + lib.rs wiring).** Delete
   `app/src/server/telemetry/` + `telemetry_ext*` + collector wiring in
   `lib.rs`. Fix the ~30 `send_telemetry_sync_*`/`send_telemetry_on_executor!`
   call sites (their macros are deleted). Keep the warp_core
   `send_telemetry_from_ctx!` macro and the warpui_core queue intact, so the ~774
   remaining calls still compile (enqueue → never drained). Rebuild.
3. **Cut crash reporting (Group C + build.rs Group G).** Delete
   `app/src/crash_reporting/`, the objc shim, remove all
   `#[cfg(feature="crash_reporting")]` blocks from `lib.rs`, delete the
   `build_and_link_sentry` machinery and the sentry scripts. Rebuild on macOS,
   Linux, Windows.
4. **Delete the features + deps (§3).** Remove `crash_reporting`, `cocoa_sentry`,
   `send_telemetry_to_file`, `record_app_active_events`,
   `global_ai_analytics_*`, `with_sandbox_telemetry`,
   `log_expensive_frames_in_sentry` from `app/Cargo.toml`; remove
   `global_ai_analytics_collection` from `default`. Drop `sentry`/`sentry-log`/
   `minidumper`/`crash-handler` deps. `cargo build` to surface any remaining
   `#[cfg(feature=...)]` islands; remove them. Regenerate `Cargo.lock`.
5. **Settings/UI cleanup (Group E).** Remove privacy toggles + analytics banner
   branches (or leave the toggles hidden — judgement call).
6. **Lazy call-site cleanup (Group B).** As AI/sharing/onboarding tasks delete
   their modules, the `send_telemetry_from_ctx!` calls vanish with them. Once
   call count → low, decide whether to also touch the tracked warp_core macro (§6).
7. **(Optional, accept tracked-crate edits) Phase 2 hardening (Group D/H + §6):**
   strip sentry from `warp_core`/`warp_logging`/`ai` Cargo + errors files, and
   simplify `channel/config.rs`. Only if a clean break is preferred over
   upstream-syncability.

---

## 6. Items that touch TRACKED-from-upstream crates (merge-conflict risk)

Per `docs/08-upstream-sync.md`, minimize edits to these. Each is avoidable in
Phase 1 by relying on feature gating:

| Tracked crate / file | What's there | Avoidable? |
|---|---|---|
| `crates/warp_core/src/telemetry.rs` | `send_telemetry_from_ctx!`/`send_telemetry_from_app_ctx!` macros (774 callers), `TelemetryEvent` trait, `EnablementState`, registry | **Yes** — leave intact; macros are inert when nothing drains the queue. Editing/deleting forces touching all 774 call sites. |
| `crates/warp_core/src/errors.rs:108-124`, `errors/anyhow.rs:28-29`, `errors/reqwest.rs` | `sentry::capture_error` / `capture_anyhow`, all `#[cfg(feature="crash_reporting")]` | **Yes** — not enabling the feature removes them at compile time. Deleting the cfg lines is a tracked edit. |
| `crates/warp_core/Cargo.toml:9,38-39` | `crash_reporting` feature + `sentry`/`sentry-log` optional deps | **Yes** — harmless if feature unused. Removing = tracked edit. |
| `crates/warp_core/src/channel/config.rs` + `channel/state.rs` | `TelemetryConfig`/`CrashReportingConfig` + accessors (already `None` in OSS) | **Yes** — leave as-is (low divergence). |
| `crates/warp_logging/Cargo.toml:22-23,36`, `src/native.rs`, `src/wasm.rs` | `sentry-log` layer behind `crash_reporting` | **Yes** — feature-gated; inert when off. |
| `crates/warpui_core/src/telemetry/**`, `app_focus_telemetry*` | in-memory event queue + focus events | **Yes** — leave as a no-op sink. |
| `crates/ai/Cargo.toml:12,37`, `crates/ai/src/telemetry.rs` | `crash_reporting`/`sentry` | Goes away with the whole `ai` crate (A-ai task). |

**Recommendation:** Do telemetry/crash removal entirely in `app/` + build/scripts
+ `app/Cargo.toml` features (all Tarp-owned). Leave the tracked crates' sentry
code feature-gated-off and their telemetry macros/queue as inert no-ops. This
achieves a telemetry-free *binary* with **zero** edits to tracked crates,
preserving upstream-syncability. Treat §4-Group D/H and the tracked-crate Cargo
edits as optional Phase-2 work to be weighed against merge-conflict cost.

---

## 7. Risks & cross-cutting integration points

- **~800 call sites** (Group B): the dominant `send_telemetry_from_ctx!` is a
  tracked-crate macro. Mass-deleting calls is a large diff; the no-op-queue
  approach defers it. Risk: if you delete the warp_core macro instead, you touch
  774 sites + a tracked crate — high churn, high conflict. Avoid.
- **`pre_sentry_errors` plumbing in `lib.rs`** (lines 842-897, 1057-1080, 1344-1347):
  an error-buffering channel woven through startup. Removing it touches the main
  startup sequence — test the boot path carefully (errors before init must still
  surface via logging, not silently drop).
- **`crash_recovery` vs `crash_reporting` confusion** (Group C): recovery is
  gated by `enable_crash_recovery` (build.rs:23, linux/windows), *not* by
  `crash_reporting`. Don't delete recovery with reporting. Verify
  `app/src/crash_recovery.rs` and `workspace/view/crash_recovery.rs` don't import
  the `crash_reporting` module before keeping them.
- **`should_collect_ai_ugc_telemetry` / `should_disable_telemetry`**
  (`app/src/server/telemetry/mod.rs:165,313,325`): `PrivacySettingsSnapshot`
  methods consumed by both telemetry and (transitively) AI-UGC settings. When
  Group A is deleted, prune unused `PrivacySettingsSnapshot` methods to avoid
  dead-code warnings; coordinate with the AI task.
- **macOS objc link step** (`build.rs:280` / `compile_sentry_objc_lib`): the objc
  crash_reporting.m is compiled into `warp_sentry_objc` and linked. Ensure no
  other objc/services code references it after removal (separate `services.m`
  compile at build.rs:43 is unrelated — keep it).
- **CI / dSYM upload**: `script/sentry_upload_dif.sh` + `sentry_create_release.sh`
  are almost certainly invoked from `.github/` release workflows; removing the
  scripts without removing the workflow steps will break release CI. Grep
  `.github/` for `sentry` before deleting.
- **`minidumper` IPC server** (`lib.rs:640-644`, `sentry_minidump.rs`): on
  Linux/Windows the app spawns itself as a minidump server subprocess. Removing
  the launch branch AND the `--minidump`-style arg handling must be done together,
  else the arg parser may route to a deleted path.
- **`heap_usage_tracking`** depends on `crash_reporting` (Cargo.toml:737); if heap
  tracking is wanted independently, decouple it from the sentry feature.

---

## 8. Quick verification commands (post-removal)

```
# No sentry symbols compiled in (app-owned):
rg -n "sentry|Sentry|minidump|crash-handler" app/src

# No telemetry egress left:
rg -n "rudder|Rudder|RudderStack|TelemetryApi|TelemetryCollector" app/src

# Tracked crates untouched (should show only feature-gated, unbuilt code):
git diff --stat crates/warp_core crates/warpui_core crates/warp_logging crates/editor

# Build the OSS terminal with no telemetry/crash features:
cargo build -p warp --features release_bundle
```
