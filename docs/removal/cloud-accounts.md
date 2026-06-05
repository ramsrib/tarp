# A2 — Cloud / Accounts / Sharing Removal Spec

Scope: remove every cloud-backend, account/auth, session-sharing, Warp-Drive,
Oz/ambient-agent, managed-secrets, and remote-server surface so Tarp is a plain
terminal that runs **fully logged-out** with no `*.warp.dev` / Firebase / RTC /
session-sharing endpoint reachable.

This is a sibling of the AI spec (`docs/removal/ai.md`) and the telemetry spec.
Auth/login is *intertwined* with AI (the agent backend authenticates the same
way), so this spec and the AI spec must land in a coordinated sequence — see
"Cross-cutting" and "Removal order". Evidence is from inspecting the repo on
2026-06-05; this is documentation only.

Reference baseline: `docs/05-removal-map.md` (Tier-A crate list, 356 auth /
253 cloud-object / 79 session-sharing file reaches in `app/src`),
`docs/03-dependencies.md` (the `session-sharing-protocol` git dep),
`docs/08-upstream-sync.md` (tracked-vs-owned split).

---

## 1. Crate-level worklist (Tier-A deletions)

All of these are `app/`-only consumers except where flagged. Workspace members
are globbed via `members = ["crates/*", "app"]` in `Cargo.toml:2-5`, so deleting
a crate = `rm -rf crates/<x>` **plus** removing its `[workspace.dependencies]`
line in the root `Cargo.toml` **plus** every `<x>.workspace = true` line in
consuming `Cargo.toml`s.

| Crate | files / LOC | `Cargo.toml` dep line | Consumers (reverse deps) | Notes |
|---|---|---|---|---|
| `cloud_objects` | 12 / 2401 | `Cargo.toml:39` | `cloud_object_models`, `cloud_object_client`, `cloud_object_persistence`, `warp_server_client`, `app` | Cloud-synced object model. |
| `cloud_object_client` | 1 / 372 | `Cargo.toml:40` | `warp_server_client`, `app` | |
| `cloud_object_persistence` | 6 / 1006 | `Cargo.toml:41` | `cloud_object_models`, `app` | Local cache of cloud objects. |
| `cloud_object_models` | 23 / 3461 | `Cargo.toml:42` | `cloud_object_client`, **`mcp`**, `warp_server_client`, `app` | Pulls `warp-workflows` (keep — re-home, see §5) and `session-sharing-protocol`. **`mcp` is itself a Tier-A AI deletion** — `mcp` consumes `StaticEnvVar`/`TransportType` from `crates/cloud_object_models/src/mcp.rs:17,42` (`crates/mcp/src/runtime.rs:12`). Both die together; no terminal-core reach. |
| `firebase` | 1 / 145 | `Cargo.toml:45` | `warp_server_client`, `app` | Firebase auth-token plumbing. |
| `warp_server_auth` | 9 / 1491 | `Cargo.toml:92` | `cloud_objects`, `warp_server_client`, `app` | Server auth (`test_user@warp.dev` test const at `crates/warp_server_auth/src/user_uid.rs:6`). Pulls `warp_managed_secrets`. |
| `warp_server_client` | 10 / 1385 | `Cargo.toml:93` | `app` only | GraphQL/server client + Firebase token refresh (`crates/warp_server_client/src/auth/session.rs`). Owns the `skip_login` feature (§4). |
| `managed_secrets` (`warp_managed_secrets`) | 10 / 1681 | `Cargo.toml:89` | `managed_secrets_wasm`, `warp_server_auth`, `app` | Pulls `warp_isolation_platform`. |
| `managed_secrets_wasm` | 1 / 118 | not a workspace dep (no root line) | none (wasm helper) | Delete the dir; not referenced from desktop build. |
| `isolation_platform` (`warp_isolation_platform`) | 6 / 374 | `Cargo.toml:86` | `managed_secrets`, `app` | Sandbox for cloud/CI agent runs. |
| `remote_server` | 22 / 8382 | `Cargo.toml:63` | **`warp_files`**, `app` | Remote-codebase / SSH host indexing. `warp_files` couples to it (see §6 — tracked-crate reach). |
| `serve-wasm` | 1 / 96 | excluded from `default-members` (`Cargo.toml:8-10`); globbed member | none | Web-compiled server helper. Safe `rm -rf`. |
| `onboarding` | 36 / 11516 | `Cargo.toml:60` | `app` only | First-run/account onboarding flows. 19 `app/src` files `use onboarding::` (list in §3). |
| `warp_web_event_bus` | 1 / 57 | `Cargo.toml:96` | **`warp_logging`**, `app` | Web event bus. `warp_logging`'s only use is in `crates/warp_logging/src/wasm.rs:174`, which is **wasm-only** (`#[cfg(target_family = "wasm")]`, gated at `crates/warp_logging/src/lib.rs:27-28`). Desktop terminal never compiles it → deletable with **zero** edits to `warp_logging`'s native path; just drop the `warp_web_event_bus.workspace = true` line from `crates/warp_logging/Cargo.toml`. Confirm via build. |

GraphQL plumbing (`graphql`, `warp_graphql_schema`) is the backend transport for
the above; after the cloud client is gone it is dead, but `graphql` is in
`default-members` (`Cargo.toml:16`) so removal is its own follow-up — assess in
the AI spec's coordination, not here.

---

## 2. Hardcoded server URLs / keys (the channel config) — TRACKED-CRATE REACH

**This is the highest-value change and it lives in a tracked-from-upstream crate
(`warp_core`).** All production endpoints and the Firebase API key are defined in
one file:

`crates/warp_core/src/channel/config.rs`:
- L59 `server_root_url: "https://app.warp.dev"`
- L60 `rtc_server_url: "wss://rtc.app.warp.dev/graphql/v2"`
- L61 `session_sharing_server_url: Some("wss://sessions.app.warp.dev")`
- L62 `firebase_auth_api_key: "AIzaSyBdy3O3S9hrdayLJxJ7mriBR4qgUaUygAs"`
- L82 `oz_root_url: "https://oz.warp.dev"`

These are surfaced through `ChannelState` accessors in
`crates/warp_core/src/channel/state.rs`:
- `firebase_api_key()` L210, `ws_server_url()` L223, `rtc_http_url()` L240,
  `session_sharing_server_url()` L253, `oz_root_url()` L263,
  `server_root_url()` L267, `workload_audience_url()` L277.

Consumed by (tracked) `crates/http_client/src/lib.rs` (`rtc_http_url()` L392,
host-allowlist comments L252-263) and `crates/websocket/src/proxy_tests.rs`.

App-side literal duplicates (Tarp-owned, delete with their modules):
- `app/src/workspace/view/openwarp_launch_modal/view.rs:30` `OZ_URL`
- `app/src/ai/agent_management/cloud_setup_guide_view.rs:37` `OZ_URL`
- `app/src/terminal/view/ambient_agent/tips.rs:57,125` `https://oz.warp.dev`

### Tracked-crate strategy (minimize divergence)
Do **not** restructure `config.rs`/`state.rs`. Make the smallest possible edit:
neutralize the endpoints in `WarpServerConfig::production()` / `OzConfig::production()`
to empties (`""` / `None` for `session_sharing_server_url`) so no network host is
ever dialed, and leave the struct shapes and accessor signatures byte-identical to
upstream. This keeps `config.rs`/`state.rs` cherry-pickable. The accessors can
stay (they'll return empty/None); callers in the cloud/auth layer are being
deleted anyway. Flag this file in `UPSTREAM_SYNC` as a known 1-hunk divergence.

(Alternative — leave the URLs and rely solely on deleting every caller — is
acceptable but riskier: a future synced commit could reintroduce a live caller.
Neutralizing at the source is the belt-and-suspenders choice.)

---

## 3. `app/`-layer worklist (the real effort)

### 3a. Directories to delete wholesale
- `app/src/auth/` — account auth UI/state. 15 files incl. `auth_manager.rs`,
  `auth_state.rs` (referenced from `lib.rs:150`), `auth_view_modal.rs`,
  `login_error_modal.rs`, `login_failure_notification.rs`, `login_slide.rs`,
  `needs_sso_link_view.rs`, `paste_auth_token_modal.rs`, `web_handoff.rs`,
  `user_properties.rs`. **NOTE: `auth_state::{AuthState, AuthStateProvider}` is
  the DI singleton wired in `app/src/lib.rs:1128,1157` and read in ~hundreds of
  files** — this is the deepest seam. See §4 strategy.
- `app/src/cloud_object/` — `breadcrumbs.rs`, `grab_edit_access_modal.rs`,
  `mod.rs`, `model/` (used at `lib.rs:253-255`), `toast_message.rs`.
- `app/src/drive/` — Warp Drive (the cloud objects UI): `panel.rs`, `index.rs`,
  `items/`, `folders/`, `import/`, `export.rs`, `sharing/` (session/object
  sharing: `dialog/`, `qr_code.rs`, `style.rs`), `workflows/` (re-home, §5),
  `cloud_action_confirmation_dialog.rs`, `cloud_object_naming_dialog.rs`,
  `cloud_object_styling.rs`, `empty_trash_confirmation_dialog.rs`, `settings.rs`.
- `app/src/external_secrets/` — managed-secrets UI (`mod.rs`).
- `app/src/server/cloud_objects/` — `listener.rs` (`lib.rs:279`),
  `update_manager.rs` (`lib.rs:280`), `fake_object_client.rs`.
- `app/src/server/server_api/` + `server_api.rs` — `ServerApiProvider`
  (`get_cloud_objects_client`/`get_managed_secrets_client`, used at
  `lib.rs:1382,1738,1821,1887`).
- `app/src/server/iap.rs` + `iap_tests.rs`, `server/sync_queue*.rs`,
  `server/network_log*.rs`, `server/graphql/`, `server/voice_transcriber.rs`
  (AI — coordinate with AI spec).
- `app/src/workspace/view/onboarding.rs`, `app/src/workspace/view/openwarp_launch_modal/`,
  `app/src/workspace/view/orchestration_launch_modal/`,
  `app/src/workspace/view/cloud_agent_capacity_modal/`,
  `app/src/workspace/view/launch_modal/oz_launch.rs`.
- `app/src/settings/onboarding.rs` (+`_tests`), `app/src/settings_view/teams_page.rs`,
  `app/src/settings_view/platform/create_api_key_modal.rs`.

### 3b. `onboarding` crate consumers to unwire (19 files)
`use onboarding::` / `onboarding::` appears in: `app/src/lib.rs` (`onboarding::init(ctx)`
at `lib.rs:1620`), `root_view.rs`, `settings/onboarding.rs`, `settings/mod.rs`,
`auth/login_slide.rs`, `workspace/one_time_modal_model.rs`, `workspace/mod.rs`,
`workspace/view.rs`, `workspace/view/onboarding.rs`, `workspace/view/vertical_tabs.rs`,
`terminal/view.rs`, `terminal/view/rich_content.rs`, `terminal/view/action.rs`,
`terminal/view/block_onboarding/onboarding_prompt_block.rs`, `server/telemetry/events.rs`,
plus AI-owned (`ai/onboarding.rs`, `ai/agent/mod.rs`, `ai/agent/telemetry.rs`).
Delete the calls and the `block_onboarding/` UI; keep the surrounding terminal
view files (tracked-shaped only in `app/`, but still Tarp-owned).

### 3c. Module declarations to drop
`app/src/lib.rs` — remove `mod cloud_object;` (L17), the `use` lines L150,
L225 (`ManagedSecretManager`), L253-255, L279-280, and the DI wiring block
L1128-1157 (auth_state singleton), L1221-1290 (cloud_objects load from sqlite),
L1382 (managed secrets client), L1620 (onboarding init), L1693-1887 (cloud-object
listeners/update managers), L2072/2126/2143 (`AuthStateProvider::as_ref`).
This file is the single biggest integration hub — plan to chip at it last, after
the modules it references are gone, so the compiler enumerates the call sites.

### 3d. `app/src/features.rs` / settings
`oz`, AI, cloud feature predicates live in `app/src/features.rs` and
`app/src/settings/ai.rs` (`AuthStateProvider` reads). Remove cloud/account
predicates; coordinate AI ones with the AI spec.

---

## 4. Cargo feature flags (`app/Cargo.toml`)

Flip these **off in `default`** first (gives a buildable shrinking target before
any deletion). All are simple `feature = []` toggles unless noted.

Cloud / sharing / drive / accounts (default-on, in the `default` block
`app/Cargo.toml` ~L461-690): `cloud_mode` (L597,939), `cloud_mode_from_local_session`
(L598,940), `cloud_mode_image_context` (L599,941), `cloud_mode_setup_v2`
(L993 → enables `cloud_mode`), `cloud_mode_input_v2` (L994), `cloud_conversations`
(L592,933), `cloud_environments` (L572,915), `cloud_object_initial_load`
(L467 → `enforce_revisions_to_cloud_objects` L689), `personal_cloud_objects`
(L777), `viewing_shared_sessions` (L490,833), `shared_with_me` (L492,837),
`session_sharing` (L811), `session_sharing_acls` (L493,812), `agent_shared_sessions`
(L462,568), `drive_objects_as_context` (L538,854), `loginless_conversion`
(L504,845), `api_key_authentication` (L558,904), `team_api_keys` (L586,936),
`warp_managed_secrets` (L582,929), `skip_firebase_anonymous_user` (L655,981),
`agent_onboarding` (L461,590), `hoa_onboarding_flow` (L650,988).

Oz / ambient / handoff / remote: `oz_identity_federation` (L602,952),
`oz_platform_skills` (L601,951), `oz_launch_modal` (L611,953), `oz_changelog_updates`
(L618,776), `oz_handoff` (L659,976), `handoff_local_cloud` (L660,977),
`handoff_cloud_cloud` (L669,995 → `cloud_mode_setup_v2`), `ambient_agents_command_line`
(L579,727), `ambient_agents_image_upload` (L580,728), `ambient_agents_rtc`
(L596,935), `scheduled_ambient_agents` (L581,729), `remote_codebase_indexing`
(L670,797 → `full_source_code_embedding`).

Login fast-path (already present — **use these as a stepping stone**):
- `skip_login` (`app/Cargo.toml:813` → `warp_server_client/skip_login`).
- `fast_dev = ["skip_login"]` (L694).
`skip_login` in `crates/warp_server_client/src/auth/session.rs:98-99` already
`bail!`s all authenticated requests, and L139 short-circuits token logic. Turning
`skip_login` on **before** deletion makes the running app behave as logged-out
(no Firebase/server calls succeed) — the safest first toggle to validate the
"runs logged-out" target, then delete.

`test-util` (L821) wires `cloud_object_client/test-util` + `warp_server_auth/test-util`
— update/remove when those crates go.

---

## 5. Re-home Workflows (keep) — entanglement with the cloud layer

`warp-workflows` (Apache-2.0, keep — terminal UX) is pulled **through**
`cloud_object_models` (a Tier-A deletion) and surfaced via `app/src/drive/workflows/`.
Before deleting `cloud_object_models`, rewire workflows to a local-only path:
add `warp-workflows` as a direct dep where the surviving workflow UI lives and
strip the cloud-object wrapper. Tracked in `docs/05-removal-map.md` Tier-C; this
spec just flags it as a **blocker** for the `cloud_object_models` delete. Do the
workflow re-home in a dedicated step, not inline with cloud removal.

---

## 6. Tracked-crate reach (merge-conflict risk — flag in UPSTREAM_SYNC)

Per the project rule, minimize edits to tracked-from-upstream terminal-core
crates. Cloud/sharing removal forces edits into these tracked crates:

1. **`crates/warp_core/src/channel/config.rs` + `state.rs` + `mod.rs`** — the
   only place server URLs/Firebase key live (§2). Smallest-edit strategy:
   neutralize the 5 literals in `config.rs:59-82`; leave struct/accessor shapes
   untouched. `channel/mod.rs:38` only mentions the `--session-sharing-server-url`
   CLI flag in a comment — optionally drop the flag handling. 1-3 hunks.
2. **`crates/warp_terminal/src/shared_session.rs`** (19 lines) +
   **`crates/warp_terminal/src/model/block_id.rs`** (`From`/`Into`
   `session_sharing_protocol::common::BlockId`/`BufferId` impls at L49-70) +
   the `pub mod shared_session;` line in `crates/warp_terminal/src/lib.rs`.
   This is the `session-sharing-protocol` git-dep reach into a tracked crate.
   Coupling is **shallow** (one tiny module + type conversions). Two clean
   options: (a) delete `shared_session.rs`, the `pub mod`, and the `block_id.rs`
   impls and drop the `session-sharing-protocol` dep — small, self-contained,
   but a recurring conflict point on sync; (b) keep the files compiling by
   stubbing — not worth it, the dep itself is being removed. Recommend (a) and
   note as a known divergence.
3. **`crates/http_client/src/lib.rs`** — uses `ChannelState::rtc_http_url()`
   (L392) and has a same-origin allowlist tuned to `app.warp.dev` (L252-263,
   tests L779-787). With URLs neutralized this becomes inert; **leave the code
   as-is** (no edit) to stay upstream-shaped — it just resolves empty origins.
4. **`crates/warp_files/src/lib.rs`** — `use remote_server::manager::RemoteServerManager`
   (L18) with real call sites at L733, L878 (`host_request_handle`). Deleting
   `remote_server` **forces** edits here. `warp_files` is `app`-only-consumed and
   Tarp-owned-ish but is shared plumbing; removing the remote-host file-read
   branches is a real change. Quantify: 2 call sites + 1 import + doc comments
   (L86,91). Gate or delete the remote-host arms of the file model.
5. **`crates/warp_server_auth/src/user_uid.rs:6`**, `crates/warp_logging`
   (wasm-only, §1) — handled by crate deletion / dep-line removal; no native edit.

Net: the unavoidable tracked-crate edits are (1) `warp_core` channel config,
(2) `warp_terminal` shared_session/block_id, (3) `warp_files` remote_server arms.
All three are small and localized; record each in `UPSTREAM_SYNC` as a deliberate
divergence so future cherry-picks expect the conflict.

---

## 7. Removal / sequencing order

Do this **after telemetry** and **interleaved with AI** (auth is the AI backend's
auth too; `server_api`/`graphql`/`voice_transcriber` are shared). Within this
surface:

1. **Flip default features off** (§4) and turn on `skip_login`. Build + run →
   confirm the terminal launches and is usable with every server call failing
   (validates the logged-out target before any deletion).
2. **Neutralize the channel URLs/key** (§2) in `warp_core/config.rs`. Build.
3. **Delete the leaf UI**: `app/src/drive/` (re-home workflows first, §5),
   `app/src/cloud_object/`, `app/src/external_secrets/`, onboarding views,
   oz/launch modals, `settings_view/teams_page.rs`, `create_api_key_modal.rs`.
4. **Delete `app/src/server/` cloud bits**: `cloud_objects/`, `server_api*`,
   `iap*`, `sync_queue*`, `network_log*`. Remove their `lib.rs` wiring
   (L1221-1290, 1382, 1693-1887).
5. **Delete `app/src/auth/`** and remove the `AuthState`/`AuthStateProvider` DI
   singleton (`lib.rs:1128,1157` + the ~hundreds of `as_ref` call sites the
   compiler now flags). This is the deepest seam — do it after 3-4 so most
   readers are already gone. Remove `onboarding::init` (L1620).
6. **Delete crates** (§1) bottom-up by reverse-dep:
   `managed_secrets_wasm` → `managed_secrets` → `isolation_platform` →
   `cloud_object_persistence` → `cloud_object_client` → `cloud_object_models`
   (after workflows re-homed; together with `mcp` from the AI spec) →
   `cloud_objects` → `warp_server_auth` → `firebase` → `warp_server_client` →
   `onboarding` → `serve-wasm` → `warp_web_event_bus` → `remote_server` (after
   `warp_files` arms removed, §6).
7. **Tracked-crate edits**: `warp_terminal` shared_session/block_id +
   `session-sharing-protocol` dep removal; `warp_files` remote_server arms.
8. **Drop git/workspace deps**: `session-sharing-protocol` (`Cargo.toml:263`),
   all workspace dep lines for deleted crates (§1), then `cargo build` to
   regenerate `Cargo.lock`. Assess `graphql`/`warp_graphql_schema` as now-dead.
9. **Update `deny.toml`/`about.toml`** for the dropped `session-sharing-protocol`
   (see `docs/04-licensing.md`).

---

## 8. Confirming "runs fully logged-out"

The codebase already models a logged-out / anonymous state, so this is
achievable, not aspirational:
- `app/src/auth/auth_manager.rs` has `is_anonymous_or_logged_out()` (L640,649),
  `create_anonymous_user` (L572), and login is **lazy/gated**:
  `attempt_login_gated_feature` (L634) only triggers auth when a login-gated
  feature is touched. The terminal proper does not require auth to start.
- `skip_login` (`warp_server_client/src/auth/session.rs:98`) already makes all
  authenticated requests fail without crashing the app.
- No tracked terminal-core crate (`editor`, `command`, `warp_terminal` minus
  `shared_session`, `warp_completer`, `vim`, `syntax_tree`) depends on
  `warp_server_*` / `firebase` / `cloud_*` — verified: the only tracked reach is
  the channel config (URLs) and the shallow shared_session module.

Acceptance test after removal: launch Tarp with no network; open blocks, run
commands, completions, vim mode, settings — all function; no `*.warp.dev` /
Firebase / RTC / `sessions.app.warp.dev` connection is attempted (verify with a
packet/DNS sniffer or the now-removed network-log pane's absence).

---

## 9. Risks

- **`AuthStateProvider` blast radius** — read in many `app/src` files (356 auth
  refs per `docs/05-removal-map.md`). Deleting the singleton triggers a large
  compile-error cascade; mitigate by feature-flagging first, deleting leaf
  consumers before the provider, and chipping `lib.rs` last.
- **Workflows regression** — if `cloud_object_models` is deleted before
  `warp-workflows` is re-homed, saved-command workflows break. §5 is a hard
  prerequisite.
- **`mcp` ↔ `cloud_object_models` coupling** — must be sequenced with the AI
  spec; neither crate can be deleted independently.
- **`warp_files` remote_server arms** — removing them changes file-model
  behavior for remote hosts; ensure local file reads are untouched.
- **Tracked-crate divergence** — the `warp_core` channel config and
  `warp_terminal` shared_session edits will conflict on future upstream syncs;
  must be logged in `UPSTREAM_SYNC` (`docs/08-upstream-sync.md`).
- **`graphql` is in `default-members`** — deleting it is a separate, larger
  follow-up; out of scope for the first cloud-removal pass.
