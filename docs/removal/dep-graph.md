# A5 — Crate Dependency Graph & Tracked-vs-Owned Split (definitive)

Reverse-dependency analysis of every removable crate, the leaf-first deletion
order, and the blockers where a **kept** crate depends on a **removable** one
(these require code edits, not just `rm`). Plus the finalized
tracked-from-upstream vs Tarp-owned path split with evidence.

**Method.** Authoritative graph built from `cargo metadata --no-deps --offline`
(72 workspace packages), edges restricted to intra-workspace deps, cross-checked
against each `crates/*/Cargo.toml`. Every claim below is reproducible:

```sh
cargo metadata --no-deps --format-version 1 --offline > /tmp/meta.json
jq -r '[.packages[].name] as $n | .packages[] | .name as $p
       | .dependencies[] | select(.name as $d | $n|index($d))
       | "\($p) -> \(.name) [kind=\(.kind//"normal") opt=\(.optional)]"' /tmp/meta.json
```

> Package-name ≠ directory for several crates. The graph keys on **package
> name**; the worklist keys on **directory**. Mapping table is in §6.

---

## 0. Scope

- **In scope:** the crate-level reverse-dependency graph for the removable set
  (AI, cloud, accounts/server, MCP, computer-use, voice, onboarding,
  managed-secrets, remote-server, isolation, web-event-bus, wasm helpers,
  GraphQL plumbing); the leaf-first deletion order; blockers into kept crates;
  the finalized tracked-vs-owned table.
- **Out of scope (other A-specs own these):** the 292 `app/Cargo.toml` feature
  flags and the `app/src` file-by-file surgery (`05-removal-map.md` + feature-flag
  spec); telemetry facade; branding. This spec governs **which crates get
  deleted, in what order, and which deletions force edits into tracked crates.**
- **Honored rules:** (1) the `editor` crate (input buffer; package `warp_editor`)
  **STAYS** — it appears in no removable edge. (2) Minimize edits to
  tracked-from-upstream terminal-core crates; every forced edit into one is
  flagged as a merge-conflict risk in §4.

---

## 1. Headline findings

1. **The terminal core is already cleanly separable at the crate-dep level.**
   `warp_terminal` and `warp_core` (the two central tracked crates) depend on
   **zero** removable crates. Their full intra-workspace dep sets are:
   - `warp_terminal` → `channel_versions, string-offset, warp_completer,
     warp_core, warp_util, warpui_core` (all kept).
   - `warp_core` → `settings, settings_value, string-offset, warp_features,
     warp_util, warpui_core, warpui_extras, websocket` (all kept).
   The entanglement the other docs describe is **`app/`-layer (package `warp`)
   and a small number of leaf edges**, not the core crates. This is the single
   most reassuring result for upstream-sync viability.

2. **Almost the entire removable set is a self-contained subgraph** whose only
   external consumer is `app` (package `warp`). Once `app`'s edges are cut
   (feature-flag + app-surgery work, owned by other specs), the removable crates
   delete cleanly leaf-first with **no further edits to tracked crates** — except
   the four blockers in §4.

3. **Two hard blockers force edits into tracked terminal crates:**
   - `warp_files → remote_server` (non-optional, real desktop code path —
     `FileBackend::Remote` at `crates/warp_files/src/lib.rs:18,732-736,878`).
     Removing `remote_server` forces an edit into `warp_files` (tracked). See §4.1.
   - **`crates/persistence` (tracked) → `warp_multi_agent_api`** (external git dep,
     so it does **not** appear in the `cargo metadata` intra-workspace graph above —
     but removing the AI dep still forces a tracked-crate source edit). Drop
     `warp_multi_agent_api.workspace = true` (`crates/persistence/Cargo.toml:23`) and
     the `api::` uses in `crates/persistence/src/model.rs:8-9` (+ `model_tests.rs:3`).
     Owned by the AI spec (`ai.md` §5.1), which calls this the highest merge-conflict
     risk of the AI removal. Listed here so the dep-graph isn't read as "only
     `warp_files` needs a tracked edit." See §4.1 (C1 reconciliation).

4. **Two "blockers" are false alarms (WASM-only / optional):**
   - `warp_logging → warp_web_event_bus` is under
     `[target.'cfg(target_family = "wasm")']` (`crates/warp_logging/Cargo.toml:25-30`)
     — **not compiled on desktop**; deletable without touching desktop builds.
   - `warp_completer → warp_js` is `optional` behind the `v2` feature
     (`crates/warp_completer/Cargo.toml:35,60`) — not a hard blocker.

5. **`graphql`/`warp_graphql_schema` are pure cloud plumbing, reachable only from
   removables + `app`** — no kept terminal crate touches them. They become
   deletable leaves once the cloud/server/ai crates and the `app` edge are gone.

6. **`lsp`/`node_runtime`/`warp_js` are an "assess, not remove" cluster** — they
   power LSP-backed completions and JS plugins, not AI/cloud. Treat as KEEP for
   v1 unless Tarp explicitly drops plugins/LSP. Do **not** put them in the core
   removal set (corrects the speculative "assess" framing in `02`/`05`).

---

## 2. Reverse-dependency graph (who depends on each removable)

Edges from `cargo metadata`. `K` = consumer is KEPT, `R` = consumer is REMOVABLE,
`app` = the binary (package `warp`). `dev`/`opt`/`wasm` annotations matter for
ordering. Only `normal`+`build` non-wasm edges block a desktop build.

| Removable crate (dir) | pkg name | Depended on by | Notes |
|---|---|---|---|
| `ai` | `ai` | `app`(normal+dev), `cloud_object_models`(R), `onboarding`(R) | top of AI subtree |
| `computer_use` | `computer_use` | `app`, `ai`(R) | |
| `voice_input` | `voice_input` | `app` (**optional**) | optional in `app` → easiest cut |
| `mcp` | `mcp` | `app` | `mcp → cloud_object_models`(R) |
| `cloud_objects` | `cloud_objects` | `app`, `warp_server_client`(R), `cloud_object_client/_models/_persistence`(R) | |
| `cloud_object_client` | `cloud_object_client` | `app`(normal+dev), `warp_server_client`(R) | |
| `cloud_object_models` | `cloud_object_models` | `app`, `cloud_object_client`(R), `mcp`(R), `warp_server_client`(R) | pulls `ai`(R) + `warp-workflows`(KEEP, re-home) |
| `cloud_object_persistence` | `cloud_object_persistence` | `app`, `cloud_object_models`(R) | |
| `firebase` | `firebase` | `app`(dev), `warp_server_client`(R) | |
| `warp_server_auth` | `warp_server_auth` | `app`(normal+dev), `cloud_objects`(R), `warp_server_client`(R) | |
| `warp_server_client` | `warp_server_client` | `app`(normal+dev) | aggregates most cloud crates |
| `graphql` | `warp_graphql` | `app`, `ai`(R), `cloud_objects`(R), `cloud_object_client/_models/_persistence`(R), `warp_managed_secrets`(R), `warp_server_auth`(R), `warp_server_client`(R) | **no KEPT consumer** |
| `warp_graphql_schema` | `warp_graphql_schema` | `warp_graphql`(R) only | 2 LOC shim; dies with `graphql` |
| `onboarding` | `onboarding` | `app` | `onboarding → ai`(R) |
| `managed_secrets` | `warp_managed_secrets` | `app`, `managed_secrets_wasm`(R), `warp_server_auth`(R) | `→ warp_graphql`(R), `→ warp_isolation_platform`(R) |
| `managed_secrets_wasm` | `managed_secrets_wasm` | — (none) | wasm helper; leaf |
| `remote_server` | `remote_server` | `app`, **`warp_files`(K)** | **BLOCKER §4.1** |
| `serve-wasm` | `serve-wasm` | — (none) | not in default-members; leaf |
| `isolation_platform` | `warp_isolation_platform` | `app`, `warp_managed_secrets`(R) | |
| `warp_web_event_bus` | `warp_web_event_bus` | `app`, **`warp_logging`(K, wasm-only)** | **soft blocker §4.2** |

Assess cluster (NOT in core removal set — keep for v1):

| Crate (dir) | pkg | Depended on by | Verdict |
|---|---|---|---|
| `node_runtime` | `node_runtime` | `lsp`(K) | KEEP — LSP server install (node) |
| `warp_js` | `warp_js` | `app`(opt), `warp_completer`(K, opt `v2`) | KEEP — JS plugins/completer |
| `lsp` | `lsp` | `app` | KEEP — language-server completions |
| `input_classifier` | `input_classifier` | `app`, — | ASSESS — only fed AI input routing; `→ natural_language_detection`(R-adjacent) |
| `natural_language_detection` | `natural_language_detection` | `app`, `input_classifier` | ASSESS — pairs with above; remove only after AI gone |

**Edges among removables (the internal subtree to delete leaf-first):**

```
warp_server_client → {cloud_objects, cloud_object_client, cloud_object_models,
                      firebase, warp_server_auth, warp_graphql}
warp_server_auth   → {cloud_objects, warp_managed_secrets, warp_graphql}
cloud_objects      → warp_server_auth
cloud_object_client→ {cloud_objects, cloud_object_models, warp_graphql}
cloud_object_models→ {cloud_objects, cloud_object_persistence, ai, warp_graphql, warp-workflows*}
cloud_object_persistence → {cloud_objects, warp_graphql}
mcp                → cloud_object_models
onboarding         → ai
ai                 → {computer_use, warp_graphql}
managed_secrets    → {warp_graphql, warp_isolation_platform}
managed_secrets_wasm → warp_managed_secrets
warp_graphql       → warp_graphql_schema
```
\* `warp-workflows` is a KEEP crate currently pulled via `cloud_object_models`.
Re-home it before deleting `cloud_object_models` (cross-spec; see `03`/`05` Tier C).

There is a dependency **cycle** among the cloud crates
(`cloud_objects ↔ warp_server_auth`, and the `cloud_object_*` mesh). They cannot
be deleted strictly one-at-a-time bottom-up; delete the **cloud/server cluster as
one batch** after `app`'s edges to them are removed (§3, Wave C).

---

## 3. Leaf-first deletion order

Precondition for every wave: the corresponding `app/` (package `warp`) edges and
default features are already removed by the feature-flag + app-surgery specs.
Crate deletion is the *last* step of each surface, not the first. After each
wave: remove the crate dir, its `members`/`workspace.dependencies` entries in
root `Cargo.toml`, any `default-members` entry, then
`cargo metadata --offline` (fast) and `cargo build` to confirm.

| Wave | Delete (dirs) | Why this order | Blocks none of / unblocked by |
|---|---|---|---|
| **A. True leaves** | `voice_input`, `serve-wasm`, `managed_secrets_wasm` | zero kept consumers; `voice_input` is `app`-optional | independent |
| **B. AI subtree** | `computer_use` → then `ai` | `ai → computer_use`; both only consumed by `app`/`onboarding`/`cloud_object_models` (handled in C) | needs `onboarding` & `cloud_object_models` app-edges cut first |
| **C. Cloud/server/onboarding batch** | `onboarding`, `mcp`, `warp_server_client`, `warp_server_auth`, `cloud_object_client`, `cloud_object_persistence`, `cloud_object_models`, `cloud_objects`, `firebase`, `managed_secrets`, `isolation_platform` | mutually-recursive cloud mesh + cycle → delete together; all rooted at `app` + the now-gone AI subtree | re-home `warp-workflows` out of `cloud_object_models` first |
| **D. GraphQL plumbing** | `graphql` (`warp_graphql`), then `warp_graphql_schema` | only consumers were Waves B/C + `app`; now dangling | last cloud leaves |
| **E. remote-server (blocked)** | `remote_server` | requires `warp_files` edit first (§4.1) | **gated on §4.1 code change** |
| **F. web-event-bus (soft)** | `warp_web_event_bus` | only desktop consumer is `app`; `warp_logging` edge is wasm-only | drop `app` edge; optionally clean wasm edge §4.2 |
| **G. assess later** | (`input_classifier`, `natural_language_detection` only if AI input-routing fully gone) | not cloud/AI-core; keep until proven dead | post-AI audit |

Approx. LOC reclaimed by wave (crate `src` line counts):
A ≈ 0.6k · B ≈ 30.1k (`ai` 25.8k + `computer_use` 4.3k) ·
C ≈ 24.3k (`onboarding` 11.0k, `cloud_object_models` 3.5k, `cloud_objects` 2.4k,
`mcp` 2.2k, `managed_secrets` 1.7k, `warp_server_auth` 1.5k,
`warp_server_client` 1.4k, `cloud_object_persistence` 1.0k,
`cloud_object_client` 0.4k, `isolation_platform` 0.4k, `firebase` 0.1k) ·
D ≈ 9.1k (`graphql` 9.1k + `warp_graphql_schema` 2 lines) ·
E ≈ 8.4k (`remote_server`) · F ≈ 0.06k. **Total ≈ 72k LOC of crate code**,
before the much larger `app/src` reduction owned by other specs.

---

## 4. Blockers — removable crates that a KEPT crate depends on

These are the only places where deleting a removable crate forces a code change
rather than a clean `rm`. Ordered by severity.

### 4.1 `warp_files (KEEP, tracked) → remote_server (REMOVE)` — HARD BLOCKER ⚠ tracked-crate edit

- Edge: non-optional, desktop (not cfg-gated). `crates/warp_files/Cargo.toml:16`
  (`remote_server.workspace = true`).
- Code: `crates/warp_files/src/lib.rs:18` `use remote_server::manager::RemoteServerManager;`
  used in the `FileBackend::Remote { host_id, path }` match arm at
  `crates/warp_files/src/lib.rs:732-736` and `:878` (`RemoteServerManager::as_ref(ctx)
  .host_request_handle(host_id)` for remote read/write).
- Meaning: `warp_files` supports two backends — local and **remote** (SSH/warp
  remote-server). The remote backend is the removable feature; the local backend
  is core terminal file handling.
- **Required change (into a tracked crate):** drop the `FileBackend::Remote`
  variant and its match arms, remove the `remote_server` import + Cargo dep from
  `warp_files`. This is the only edit to a tracked terminal-core crate the whole
  crate-removal effort demands.
- **Merge-conflict risk: HIGH.** `warp_files` is tracked-from-upstream; any
  upstream change to `FileBackend` / save logic will conflict. Mitigation: keep
  the edit **minimal and localized** (delete the enum variant + its arms, nothing
  else); document it in `UPSTREAM_SYNC` as a known divergence; gate behind the
  same feature flag the app uses for remote-server so the diff stays mechanical.
- Sequencing: this edit is the precondition for Wave E.

### 4.2 `warp_logging (KEEP, tracked) → warp_web_event_bus (REMOVE)` — SOFT (wasm-only)

- Edge under `[target.'cfg(target_family = "wasm")']`
  (`crates/warp_logging/Cargo.toml:25-30`); usage only in
  `crates/warp_logging/src/wasm.rs`. **Not compiled in the desktop build.**
- Action: `warp_web_event_bus` can be deleted for desktop without touching the
  desktop path. The cleanest end-state still removes the wasm dep line + `wasm.rs`
  reference — a tiny tracked-crate edit, low conflict risk, only needed if the
  wasm target is also being dropped (likely yes, since serve-wasm is removed).
- Merge-conflict risk: LOW (wasm block rarely changes; edit is one dep line).

### 4.3 `warp_completer (KEEP, tracked) → warp_js (KEEP-for-now)` — NOT a blocker

- `crates/warp_completer/Cargo.toml:35` `warp_js = { workspace = true, optional
  = true }`, gated by feature `v2` (`:60 v2 = ["dep:warp_js", "dep:rquickjs"]`).
- Since `warp_js` is in the KEEP/assess cluster, no action. If `warp_js` is ever
  removed, just drop the `v2` feature — no forced edit. Documented for
  completeness.

### 4.4 `lsp (KEEP) → node_runtime (KEEP)` — NOT a removal blocker

- `crates/lsp/Cargo.toml:28` non-optional; used in
  `crates/lsp/src/servers/{pyright,typescript_language_server}.rs`, `install.rs`.
- Both crates are KEEP (LSP completion infra, not AI/cloud). No action for v1.

**No blocker exists into `warp_terminal`, `warp_core`, `command`, `editor`,
`warp_completer` (beyond the optional `warp_js`), `vim`, `syntax_tree`,
`languages`, or the `warpui*` renderer crates.** The crate graph confirms the
removable subgraph is detachable with exactly one mandatory tracked-crate edit
(§4.1) plus one optional cleanup (§4.2).

---

## 5. Definitive Tracked-vs-Owned table (refines `08`)

Classification driver: a crate is **TRACKED** (keep upstream-shaped, sync via
cherry-pick) if it is terminal-core and depends on no removable crate (or only
the §4 edges, which we keep minimal). It is **OWNED** (accept divergence /
delete) if it is part of the removable surface or is `app/` itself.

| Crate (dir) | pkg name | Class | Depends on removable? | Evidence / note |
|---|---|---|---|---|
| `warpui` | `warpui` | TRACKED | no | MIT renderer |
| `warpui_core` | `warpui_core` | TRACKED | no | MIT |
| `warpui_extras` | `warpui_extras` | TRACKED | no | |
| `ui_components` | `ui_components` | TRACKED | no | |
| `warp_terminal` | `warp_terminal` | TRACKED | **no** | deps all kept (§1) |
| `warp_core` | `warp_core` | TRACKED | **no** | deps all kept (§1) |
| `command` | `command` | TRACKED | no | |
| `editor` | `warp_editor` | TRACKED | no | **input buffer — STAYS** |
| `warp_completer` | `warp_completer` | TRACKED | only `warp_js` (opt, `v2`) | §4.3 |
| `command-signatures-v2` | `command-signatures-v2` | TRACKED | no | completions |
| `vim` | `vim` | TRACKED | no | |
| `syntax_tree` | `syntax_tree` | TRACKED | no | |
| `languages` | `languages` | TRACKED | no | |
| `markdown_parser` | `markdown_parser` | TRACKED | no | |
| `fuzzy_match` | `fuzzy_match` | TRACKED | no | |
| `sum_tree` | `sum_tree` | TRACKED | no | |
| `string-offset` | `string-offset` | TRACKED | no | |
| `warp_util` | `warp_util` | TRACKED | no | |
| `settings` / `settings_value` / `settings_value_derive` | same | TRACKED | no | |
| `warp_features` | `warp_features` | TRACKED | no | flag plumbing |
| `persistence` | `persistence` | TRACKED | **no** (intra-workspace) | AI touch is via *types/app*, not a crate dep |
| `warp_search_core` | `warp_search_core` | TRACKED | no | |
| `warp_ripgrep` | `warp_ripgrep` | TRACKED | no | |
| `warp_cli` | `warp_cli` | TRACKED | no | |
| `channel_versions` | `channel_versions` | TRACKED | no | |
| `warp_assets` / `asset_cache` / `asset_macro` | same | TRACKED | no | |
| `virtual_fs` | `virtual-fs` | TRACKED | no | |
| `watcher` | `watcher` | TRACKED | no | |
| `repo_metadata` | `repo_metadata` | TRACKED | no | |
| `prevent_sleep` / `app-installation-detection` | same | TRACKED | no | OS utils |
| `http_client` / `http_server` / `websocket` / `ipc` / `jsonrpc` / `field_mask` | same | TRACKED | no | transport plumbing |
| `handlebars` | `handlebars` | TRACKED | no | vendored fork |
| `simple_logger` | `simple_logger` | TRACKED | no | |
| `warp_logging` | `warp_logging` | TRACKED | only wasm `warp_web_event_bus` | §4.2 |
| `warp_files` | `warp_files` | TRACKED ⚠ | **yes — `remote_server`** | **§4.1 forced edit** |
| `lsp` | `lsp` | TRACKED (assess) | `node_runtime` (kept) | keep for v1 |
| `node_runtime` | `node_runtime` | TRACKED (assess) | no | LSP install |
| `warp_js` | `warp_js` | TRACKED (assess) | no | JS plugins |
| `input_classifier` | `input_classifier` | OWNED (assess) | `natural_language_detection` | remove post-AI |
| `natural_language_detection` | `natural_language_detection` | OWNED (assess) | no | remove post-AI |
| `integration` | `integration` | OWNED | (test harness; pulls removables in dev) | strip AI/cloud tests |
| `ai` | `ai` | **OWNED — DELETE** | yes | Wave B |
| `computer_use` | `computer_use` | **OWNED — DELETE** | — | Wave B |
| `voice_input` | `voice_input` | **OWNED — DELETE** | no | Wave A |
| `mcp` | `mcp` | **OWNED — DELETE** | `cloud_object_models` | Wave C |
| `cloud_objects` | `cloud_objects` | **OWNED — DELETE** | yes | Wave C |
| `cloud_object_client` | `cloud_object_client` | **OWNED — DELETE** | yes | Wave C |
| `cloud_object_models` | `cloud_object_models` | **OWNED — DELETE** | yes (+ re-home `warp-workflows`) | Wave C |
| `cloud_object_persistence` | `cloud_object_persistence` | **OWNED — DELETE** | yes | Wave C |
| `firebase` | `firebase` | **OWNED — DELETE** | no | Wave C |
| `warp_server_auth` | `warp_server_auth` | **OWNED — DELETE** | yes | Wave C |
| `warp_server_client` | `warp_server_client` | **OWNED — DELETE** | yes | Wave C |
| `graphql` | `warp_graphql` | **OWNED — DELETE** | `warp_graphql_schema` | Wave D — **no kept consumer** |
| `warp_graphql_schema` | `warp_graphql_schema` | **OWNED — DELETE** | no | Wave D (2-LOC shim) |
| `onboarding` | `onboarding` | **OWNED — DELETE** | `ai` | Wave C |
| `managed_secrets` | `warp_managed_secrets` | **OWNED — DELETE** | yes | Wave C |
| `managed_secrets_wasm` | `managed_secrets_wasm` | **OWNED — DELETE** | `warp_managed_secrets` | Wave A leaf |
| `remote_server` | `remote_server` | **OWNED — DELETE** | no | Wave E (gated §4.1) |
| `serve-wasm` | `serve-wasm` | **OWNED — DELETE** | no | Wave A leaf |
| `isolation_platform` | `warp_isolation_platform` | **OWNED — DELETE** | no | Wave C |
| `warp_web_event_bus` | `warp_web_event_bus` | **OWNED — DELETE** | no | Wave F (§4.2) |
| `app` | `warp` | **OWNED** | yes (everything) | the surgery target — feature-flag spec owns it |

**Corrections to prior docs captured here:**
- `08` listed `firebase`, `warp_server_*`, `managed_secrets*`, `remote_server`,
  `serve-wasm`, `isolation_platform`, `warp_web_event_bus` as owned — confirmed,
  and now backed by reverse-dep evidence with package-name disambiguation.
- `08`/`02`/`05` left `node_runtime`/`warp_js`/`lsp`/`input_classifier`/
  `natural_language_detection` as "assess." Refined: `lsp`/`node_runtime`/
  `warp_js` are TRACKED-keep (LSP/plugins, not AI/cloud); only
  `input_classifier`/`natural_language_detection` are OWNED-assess (post-AI).
- `02` calls `persistence` "also touches multi-agent API" — true at the
  *type/app* level, but `persistence` has **no intra-workspace crate dep on any
  removable** (its `warp_multi_agent_api` usage is the external git dep, handled
  in the deps spec). So `persistence` stays TRACKED; the multi-agent unwiring is
  `app/`-layer + dep-pruning, not a crate deletion.

---

## 6. Package-name ↔ directory map (needed for edits)

`cargo metadata` keys on package name; `crates/<dir>` and root `Cargo.toml`
`members`/`workspace.dependencies` key on directory/alias. Mismatches:

| Directory | Package name | workspace.dependencies alias |
|---|---|---|
| `crates/editor` | `warp_editor` | `editor` (`warp_editor` = `{ path = "crates/editor" }`) |
| `crates/graphql` | `warp_graphql` | `warp_graphql = { path = "crates/graphql" }` |
| `crates/managed_secrets` | `warp_managed_secrets` | `warp_managed_secrets` |
| `crates/isolation_platform` | `warp_isolation_platform` | `warp_isolation_platform` |
| `crates/virtual_fs` | `virtual-fs` | `virtual-fs` |
| `crates/serve-wasm` | `serve-wasm` | (not in default-members) |

When deleting a crate, remove **all three**: the `crates/<dir>` directory, its
`[workspace.dependencies]` line in root `Cargo.toml`, and any `default-members`
entry — then regenerate `Cargo.lock` (`cargo update -w --offline` or build).

---

## 7. Cross-cutting integration points (crate-graph level)

- **`app` (package `warp`)** is the universal consumer: it has direct
  non-optional edges to nearly every removable crate (§2 full list). All crate
  deletions are gated on the `app/`-layer feature-flag + surgery work (other
  specs). This spec's order assumes those `app` edges are cut wave-by-wave.
- **`integration`** (test harness) pulls removables as dev-deps; its AI/cloud
  tests must be stripped in lockstep with each wave or `cargo test` breaks. Not a
  desktop-build blocker (not in default-members).
- **`warp-workflows` (KEEP)** enters the graph only through `cloud_object_models`
  (Wave C). Re-home it (local-only workflows) **before** Wave C or workflows die
  with the cloud cluster. Cross-ref `05` Tier C / deps spec.
- **`default-members`** currently includes `graphql` and `editor` among others
  (`Cargo.toml:11-22`). Removing `graphql` requires editing `default-members`;
  `editor` stays. No other removable crate is in `default-members`.
- **Dependency cycle** in the cloud cluster (`cloud_objects ↔ warp_server_auth`,
  `cloud_object_*` mesh) means Wave C is an atomic batch, not incremental.

---

## 8. Risks

1. **§4.1 `warp_files` edit is unavoidable and recurring-conflict-prone.** It is
   the one tracked-core edit the crate plan forces. Keep it surgical and flag it
   in `UPSTREAM_SYNC`. If upstream refactors `FileBackend`, expect a manual
   re-apply each sync.
2. **Cloud cycle batch (Wave C) is all-or-nothing** — a half-deleted cluster
   won't compile. Do the whole batch + `app` edge removal in one branch.
3. **`warp-workflows` orphaning** — if Wave C runs before workflows are
   re-homed, a KEEP feature is lost. Hard-sequence the re-home first.
4. **Assess-cluster premature removal** — deleting `input_classifier`/`nld`/
   `node_runtime`/`warp_js`/`lsp` before confirming the terminal doesn't use them
   would regress completions/plugins/LSP. Keep them until a post-AI audit proves
   them dead.
5. **Package-name traps** — grepping by directory name misses
   `warp_graphql`/`warp_managed_secrets`/`warp_isolation_platform`/`warp_editor`/
   `virtual-fs`. Use §6 when wiring deletions or the manifest edits will be
   incomplete and the build will still link the crate.
6. **wasm target** — `managed_secrets_wasm`, `serve-wasm`, `warp_web_event_bus`,
   and `warp_logging::wasm` only matter if the wasm/remote-web target is kept.
   Tarp drops it, so they're free leaves — but confirm no CI job builds the wasm
   target before deleting (CI spec owns that check).
