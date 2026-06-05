# 00 — Completeness Review of the 8 Removal Specs (Wave 1 critic pass)

Reviewer role: completeness critic. Inputs read in full: the 8 specs in
`docs/removal/` (`ai.md`, `cloud-accounts.md`, `telemetry.md`, `code-editor.md`,
`feature-flags.md`, `dep-graph.md`, `branding-map.md`, `ci-plan.md`) plus
`docs/05-removal-map.md` and `docs/08-upstream-sync.md`. Several load-bearing
claims were spot-verified against the live repo at commit `2bb3a04b` (see the
"Verified" markers below). This is analysis only; no source or spec was edited.

The headline: **the spec set is unusually coherent and execution-ready.** Coverage
of the removable surface is near-complete, the tracked-vs-owned discipline is
consistently applied, and the deletion orders agree across specs. There are a
handful of real conflicts to reconcile and three coverage seams to close before
Wave 2 starts — none of them blocking, all of them cheap.

---

## 1. Status table of the 8 specs

Spec-ID column uses the labels the specs assign themselves (A1–A8); note the
filename↔A-number mapping is **not** in numeric order (see Gap G1).

| File | Self-label | Surface | Completeness | Internal order | Tracked-crate discipline | Status |
|---|---|---|---|---|---|---|
| `ai.md` | A1-ai | AI / agent / MCP / computer-use / voice / classifiers | High — file-level, sequenced | Clear (6 steps) | Excellent (persistence flagged as the one real edit) | **READY** |
| `cloud-accounts.md` | A2 | Cloud / accounts / sharing / drive / oz / secrets / remote-server | High | Clear (9 steps) | Excellent (channel config + warp_files + shared_session flagged) | **READY** |
| `telemetry.md` | A3 | Telemetry / analytics / Sentry / crash-reporting | High — best two-phase analysis | Clear (7 steps) | Excellent (zero tracked edits in Phase 1) | **READY** |
| `code-editor.md` | A4 | Warp Code cluster (editor / file-tree / diff / PR review / LSP) | High | Clear (8 steps) | Excellent (editor-crate disambiguation explicit) | **READY** |
| `dep-graph.md` | A5 | Crate reverse-dep graph + leaf-first deletion order | High — reproducible from `cargo metadata` | Clear (Waves A–G) | Strong (one forced edit: `warp_files`) | **READY w/ 1 fix** (persistence classification, C1) |
| `branding-map.md` | A6 | Warp→Tarp branding, channels, assets, URLs, env vars | High — file:line grade | Clear (10 steps) | Strong (channel enum + paths + bootstrap flagged) | **READY** |
| `ci-plan.md` | A7 | `.github/` + `script/` CI/release | High — drafts included | Clear (6 steps) | N/A (zero tracked-crate edits, correctly) | **READY** |
| `feature-flags.md` | A8 | All 292 cargo features → KEEP/REMOVE + minimal `default` | High — verbatim `default` block | Clear (7 steps) | Strong (RELEASE_FLAGS caveat is the subtle bit) | **READY w/ caveats** (C2, G2) |

No spec is "not ready." The two "w/" markers are reconciliations, not rewrites.

---

## 2. Coverage gaps

### G1 (cosmetic, fix before execution) — A-number / filename mismatch
The specs cross-reference each other by A-number, but the numbering is inconsistent
with the filenames and with `docs/09-parallel-execution-plan.md`'s own scheme:
- `ai.md` = "A1", `telemetry.md` = "A3", `code-editor.md` = "A4",
  `dep-graph.md` = "A5", `branding-map.md` = "A6", `ci-plan.md` = "A7",
  `feature-flags.md` = "A8" — but `cloud-accounts.md` = "A2".
- A4 (code-editor) repeatedly refers to "the AI spec (A?-ai)" and "A5/the
  removal-map track" for feature defaults, while A8 (feature-flags) is the spec
  that actually owns the `default` set. A4 §0 says "Cloud … removed with the cloud
  spec," A2 says it is "A2," consistent — but A8 §7 says crate deletions follow
  "A5/the removal-map track," conflating A5 (dep-graph) with doc 05.
- **Fix:** add a one-line legend mapping {A1=ai, A2=cloud-accounts, A3=telemetry,
  A4=code-editor, A5=dep-graph, A6=branding, A7=ci, A8=feature-flags} to
  `docs/removal/README.md`, and have each spec cite the **filename** not the
  A-number. Pure hygiene; no analysis changes.

### G2 (real) — feature ownership "between two specs" — the largest seam
Several features are claimed (or flagged "coordinate") by two specs without a
single declared owner. If both flip them and both delete the gated code, fine; the
risk is the opposite — **each assumes the other owns it and neither removes it.**
Concretely:
- `cloud_conversations` — A1 §3b ("leave ownership to the cloud spec, but code
  lives under `app/src/ai`, so it falls out here") and A2 §4 (lists it in the
  cloud default-off set). Ambiguous owner for the *code deletion*.
- `agent_shared_sessions` — A1 §3b and A2 §4 both list it.
- `markdown_mermaid` — A1 §3b note flags it for "the deps spec" because it pulls
  `mermaid-to-svg`, but **A8 §3a lists `markdown_mermaid` as KEEP and puts it in
  the new `default` block** (calling it "a terminal feature, not AI"). This is a
  **direct disagreement** (see C3) and also a coverage seam: who decides the
  `mermaid-to-svg` git dep's fate? Neither spec fully owns it.
- The PR-comment / diff-as-context cluster (`pr_comments_skill`, `pr_comments_v2`,
  `diff_set_as_context`, `ai_context_menu_code`, `selection_as_context`) — A4 §1b
  and A8 §3b/§3e both list them, A4 explicitly says "de-conflict ownership during
  sequencing; flip them off in whichever spec lands last." That punt is acceptable
  but needs a single named owner per feature in the execution plan.
- `global_ai_analytics_collection`/`_banner` — A1 §3 (awareness), A3 §3 (owner),
  A8 §3b/§3f (REMOVE). A3 correctly claims ownership; just confirm A1/A8 defer.
- **Fix:** add a "feature ownership ledger" (one row per contested feature →
  single owning spec for *code deletion*, separate from who flips the default).
  `docs/09-parallel-execution-plan.md` is the natural home. ~15 rows.

### G3 (real) — surfaces named in doc 05 but not clearly owned by any spec
- **`graphql` / `warp_graphql_schema`.** A2 §1 explicitly defers them ("its own
  follow-up — assess in the AI spec's coordination, not here"), A1 does **not**
  pick them up, and A5 (dep-graph) is the only spec that gives them a concrete home
  (Wave D). **Verified:** `graphql` IS in `default-members` (`Cargo.toml:15`), so
  deleting it requires a `default-members` edit, which A5 §7 notes but no `app/`
  surgery spec owns. **Net:** A5 is the de-facto owner; make that explicit so the
  `default-members` edit isn't orphaned.
- **`crates/integration` test surgery.** A1 §2 deletes `agent_mode.rs`; A5 §7 says
  "strip AI/cloud tests in lockstep." But no spec enumerates the *cloud/auth*
  integration tests the way A1 enumerates the agent one. Low risk (tests, not
  default-members), but the cloud-test strip is currently unowned in file-level
  detail.
- **`crates/lsp` vs the "assess cluster."** A4 §7 decisively says "delete
  `crates/lsp`" once Warp Code + AI are gone. A5 §1/§5 classifies `lsp` as
  **TRACKED (assess) — KEEP for v1**. This is a genuine disagreement (see C4), and
  it means the *fate of `lsp`* (and transitively `node_runtime`/`warp_js`) is
  owned by two specs with opposite verdicts.
- **`app/src/server/graphql/`, `sync_queue*`, `network_log*`, `iap*`.** A2 §3a
  lists these for deletion — good, covered. No gap.
- **`crates/warp_cli` agent/mcp subcommands.** A1 §5.5 owns these. A6 also touches
  `warp_cli` for help-text URLs. Covered, just coordinate (both edit the crate).

### G4 (minor) — items mentioned but with no acceptance/verification owner
- A3 has excellent post-removal verification commands (§8). No other spec has an
  equivalent "prove it's gone" check. A2 §8 has an acceptance test (runs
  logged-out, no `*.warp.dev` dialed) — good. **Gap:** there is no consolidated
  "Wave-complete acceptance gate" (build + run + no-network + grep-clean) tying the
  per-spec checks together. The CI `oss-build-check` (A7 §3) is the compile gate
  but not the behavioral one. Recommend one consolidated acceptance checklist in
  the execution plan.

---

## 3. Overlaps / contradictions to fix

### C1 (must fix) — `crates/persistence` classification disagreement
**A1 §5.1 and A5 §5 say different things about the same tracked crate.**
- A1 §5.1: "`crates/persistence` (TRACKED) — hard entanglement. The one real
  conflict." Requires dropping the `warp_multi_agent_api` dep + the `api::` type
  uses in `model.rs`. Correct.
- A5 §5 / §7: "`persistence` stays TRACKED; … `persistence` has **no
  intra-workspace crate dep on any removable** … the multi-agent unwiring is
  `app/`-layer + dep-pruning, **not a crate deletion**."

**Verified against the repo:**
- `crates/persistence/Cargo.toml:23` → `warp_multi_agent_api.workspace = true`
- `crates/persistence/src/model.rs:8` → `use warp_multi_agent_api::response_event::stream_finished;`
- `crates/persistence/src/model.rs:9` → `use warp_multi_agent_api::{self as api};`
- `crates/persistence/src/model_tests.rs:3` → `use warp_multi_agent_api as api;`

**Both are technically correct but A5's framing is misleading.** A5 is right that
`warp_multi_agent_api` is an *external git dep*, not an intra-workspace crate, so
it doesn't appear in A5's `cargo metadata` graph — which is why A5 cleanly
classifies `persistence` as "no removable dep." But that graph-purity hides a
**forced tracked-crate source edit**: removing the `warp_multi_agent_api` git dep
(A1 §4 / Cargo.toml:321) *requires* editing `crates/persistence/Cargo.toml` and
`model.rs`. A1 correctly calls this "the single highest merge-conflict risk in the
AI removal."
- **Fix:** A5 must add `persistence` to its blocker list (§4) as a
  **forced tracked-crate edit driven by an external-dep removal** (parallel to
  §4.1 `warp_files → remote_server`), cross-referencing A1 §5.1. Otherwise the
  dep-graph spec reads as if zero tracked edits are needed beyond `warp_files`,
  which is false. This is the one substantive correctness fix in the set.

### C2 (should reconcile) — A8's `default` count vs doc 05 / A1 / A4
- A8 §1/§4 states the current `default` has **187 entries** and reduces to ~50.
  **Verified: exactly 187.** A8 is precise.
- Doc 05 and A1 §3 both say "~190." A4 §1 cites "lines 503–676" for the default
  array; A8 cites "lines 71–260." These line-number disagreements suggest the
  specs were written against slightly different mental models of the file or at
  different times.
- **Fix:** treat A8's 187 / lines 71–260 as authoritative (it's the spec that owns
  the block and it matches the repo). Have A1 §3 and A4 §1 cite A8 for the line
  range rather than carrying their own (drifting) numbers. Not blocking — A8's
  verbatim block (§6) is the thing that actually gets applied.

### C3 (must decide — product call) — `markdown_mermaid`: KEEP or REMOVE?
- A8 §3a: **KEEP**, in the new `default` ("Mermaid diagram rendering in block
  output (terminal feature, not AI)").
- A1 §3b note: flags it for the deps spec because `markdown_mermaid` pulls the
  `mermaid-to-svg` git dep "used mainly for AI markdown output."
- These aren't strictly contradictory (a feature can be terminal-relevant *and*
  pull a heavy dep), but the **end state differs**: A8 keeps the dep alive in the
  default binary; A1 implies it may be droppable with AI. Someone must decide
  whether mermaid rendering is a terminal feature Tarp ships by default.
- **Fix:** make this an explicit product decision in the execution plan. If KEEP
  (A8's stance), A1's note should be downgraded to "no action — mermaid stays."
  If the `mermaid-to-svg` git dep is unwanted, A8 must move `markdown_mermaid` out
  of `default` and into REMOVE. Recommend defaulting to A8 (KEEP) unless the dep
  audit (doc 03) says the git dep is problematic.

### C4 (must decide) — `crates/lsp` (and node_runtime/warp_js) fate
- A4 §7: **DELETE `crates/lsp`** ("once Warp Code + AI are removed, `lsp` has no
  consumers"). A4 §7 enumerates 22 app consumers, all in the removal set.
- A5 §1.6 / §5: **KEEP for v1** ("`lsp`/`node_runtime`/`warp_js` are TRACKED-keep
  (LSP/plugins, not AI/cloud)").
- This is a real contradiction. A4 argues from *consumers* (all removed → dead),
  A5 argues from *category* (LSP completions are a terminal feature worth keeping).
  The crux: **does Tarp want LSP-backed completions/diagnostics in the terminal,
  independent of the Warp Code editor?** A4's consumer analysis suggests every
  current `lsp` consumer is in a removed surface, which would make A5's "keep"
  produce a dead crate with no callers.
- **Fix:** resolve before Wave 2 touches either. Recommended: trust A4's
  consumer-level evidence — if literally all 22 consumers are removed, `lsp` is
  dead and KEEP-ing it ships an uncalled crate. But verify A5's implicit claim that
  *some* terminal completion path uses `lsp` outside the editor; if none does, A4
  wins and `lsp`/`node_runtime` go (A5's §5 table must flip). If a terminal LSP
  path exists, A5 wins and A4 §7 must be softened to "rewire, don't delete." This
  needs a 30-minute `rg` audit, not a guess.

### C5 (minor overlap, already self-managed) — `app/src/persistence/sqlite.rs`
Both A4 §4 (code/code-review pane snapshots) and A1 §5.1 (agent conversation
tables) touch the **same Tarp-owned** restore file and the **same tracked**
`crates/persistence` schema. Both correctly say "don't edit the tracked schema,
only the app-side restore arms." No conflict — but they edit the same two files,
so the execution plan must sequence them (or merge the persistence-restore edits
into one step) to avoid two PRs fighting over `sqlite.rs`.

### C6 (minor) — `input_classifier` ownership is consistent but the verdict is a product call in two places
A1 §1 (REMOVE, "confirm with product"), A5 §G/§5 (OWNED-assess, "remove post-AI"),
doc 05 Tier C ("assess"). All agree it's AI-adjacent and removed after AI. No
contradiction; just ensure the "product confirmation" gate (A1 §8) is recorded
once, not assumed by each spec.

---

## 4. Is the minimal `default` (A8) coherent with A1/A2/A4 removals?

**Yes, with two caveats.** Cross-checking A8's verbatim `default` block (§6, ~50
entries) against what A1/A2/A4 delete:

- **No A8-kept default feature gates an A1/A2/A4-removed surface that I can find**,
  *except* the two flagged below. A8's kept set is genuinely terminal: rendering,
  input/selection, classic completions, workflows, tabs/panes, SSH, history,
  profiles, app-lifecycle. None of these names appear in A1/A2/A4's REMOVE lists.
- **A8 correctly excludes everything A1/A2/A4 remove** from the new default (agent_*,
  cloud_*, code_*, mcp_*, telemetry). The three REMOVE buckets in A8 §3b–3f line up
  with A1 (AI/MCP), A2 (cloud/accounts/oz), A4 (code-editor), A3 (telemetry).
- **A8's "shrink default first" strategy is exactly the strategy A1 §3, A2 §4,
  A3 §5, A4 §1 all assume.** This is the connective tissue — A8 is the keystone and
  the others build on it. Coherent.

Caveats:
1. **`markdown_mermaid` (C3)** is in A8's default but flagged AI-adjacent by A1.
   If A1's view wins, A8's default must drop it. (Product decision.)
2. **`command_palette_file_search` and `global_search`** are in A8's default. A4 §4
   removes the *code-pane-opening path* from `search/command_palette/files/` and
   `search/files/icon.rs`. A8 §8 itself warns "features like `global_search`,
   `command_palette_file_search` … may reference AI/cloud context types in
   non-gated code." So these are kept-by-default features that A4 partially edits.
   This is fine (A4 prunes the code-pane arm, the feature survives for plain file
   search) — but it's exactly the "first post-§6 build surfaces a non-gated edge"
   risk A8 flags. **Action:** verify these two compile after A1/A4 land; demote
   from default if not. Already anticipated by A8 §8, so not a gap — just a
   must-run check.

A8's own §4 closing note ("validate each kept default against its `cfg!`/
`FeatureFlag` site after the first build … if a kept feature fails to compile once
AI is gone, demote it") is the correct safety valve and makes the coherence
self-healing.

---

## 5. Do any specs propose edits to tracked-from-upstream crates that should be reconsidered?

The specs are disciplined here; almost every tracked-crate edit is either
**unavoidable-and-flagged** or **explicitly avoided**. Inventory of every proposed
tracked-crate edit and my verdict:

| Tracked crate / file | Proposed by | Edit | Verdict |
|---|---|---|---|
| `crates/persistence` (Cargo.toml + model.rs) | A1 §5.1 | drop `warp_multi_agent_api` dep + `api::` uses | **Unavoidable** (Verified). Keep minimal (don't delete tables). A5 must also list it (C1). |
| `crates/warp_core/src/channel/config.rs` (URLs/Firebase key) | A2 §2 | neutralize 5 literals to empty | **Reasonable** — belt-and-suspenders. Alternative (delete callers only) is acceptable; A2 already presents both. Keep as 1-hunk, log in UPSTREAM_SYNC. |
| `crates/warp_terminal` shared_session.rs + block_id.rs | A2 §6.2 | delete shared_session module + session-sharing-protocol impls | **Unavoidable** if the git dep is removed. Shallow. OK. |
| `crates/warp_files/src/lib.rs` (`FileBackend::Remote`) | A2 §6.4, A5 §4.1 | delete remote backend arm | **Unavoidable** — the one hard blocker. Both specs agree; keep surgical. |
| `crates/warp_core/src/channel/{mod,state}.rs` (channel strings) | A6 §3 | rename Oss arms to "tarp", AppId fallback, url_scheme | **Reconsider scope** — A6 itself recommends the low-divergence path (touch only Oss arms, keep enum). Endorse that; do NOT trim the Channel enum (A6's own caveat). |
| `crates/warp_core/src/paths.rs` + paths_tests.rs | A6 §3 | data-dir name mapping WarpOss→Tarp | **Necessary** for rename, but it's a behavioral change (data-dir migration). A6 §Risks notes it. Acceptable for a fresh fork. |
| `crates/warpui/src/rendering/wgpu/resources.rs` | A6 (URLs) | one `docs.warp.dev` GPU-help URL | **Optional** — cosmetic. Isolate as its own commit, or leave (it's a help link). Lowest priority. |
| `crates/warp_core/errors.rs`, `warp_logging`, channel telemetry config | A3 §6/§H | **explicitly NOT edited in Phase 1** | **Correct** — A3 leaves all tracked sentry code feature-gated-off. Exemplary. |
| `crates/warp_features` FeatureFlag enum | A1/A4/A8 | **explicitly NOT edited** (leave variants) | **Correct** — all three specs agree: gate via `app/src/features.rs`, never edit the tracked enum. Exemplary. |
| `crates/warp_core` icons.rs / context_flag / execution_mode (agent variants) | A1 §5.2–5.4 | **explicitly NOT edited** (leave inert) | **Correct.** |

**One thing to reconsider:** the `RELEASE_FLAGS::CrashReporting` handling (A8 §5,
§8). `RELEASE_FLAGS` lives in tracked `warp_features` and force-enables
`CrashReporting` in release bundles via `app/src/features.rs:22`. A8 correctly says
handle it in the **Tarp-owned bridge** (`app/src/features.rs`), not by editing
`warp_features`. **This is the subtlest tracked-crate trap in the whole set** and
A8 nails it — but A3 (telemetry, the spec that actually cares about Sentry being
off in release bundles) does **not** mention `RELEASE_FLAGS` at all. **Fix:** A3
should cross-reference A8 §5/§8 so the release-bundle telemetry re-entry path isn't
missed when A3 is executed standalone. Otherwise a release build could silently
re-enable crash reporting. (Verification welcome: confirm `RELEASE_FLAGS` contents
and the `features.rs:22` extend.)

Net: **no tracked-crate edit in the set is gratuitous.** The only correction is
A5 must acknowledge the `persistence` edit (C1); everything else is either forced,
flagged, or correctly avoided.

---

## 6. Is the A5 deletion order consistent with the per-area removal orders?

**Yes — strongly consistent, with the persistence caveat (C1) and one
sequencing-emphasis difference.**

Cross-checking A5's Waves A–G against the per-spec internal orders:

- **A5 Wave A (true leaves: `voice_input`, `serve-wasm`, `managed_secrets_wasm`)**
  ↔ A1 §7.1 deletes `voice_input` early; A2 §6 deletes `managed_secrets_wasm`
  first. Consistent.
- **A5 Wave B (`computer_use` → `ai`)** ↔ A1 §7.1–7.3 (MCP/computer_use/voice
  first, then AI core). Consistent; A5's "computer_use before ai" matches A1's
  ordering.
- **A5 Wave C (cloud/server/onboarding atomic batch, due to the
  `cloud_objects ↔ warp_server_auth` cycle)** ↔ A2 §6 deletes the cloud crates
  "bottom-up by reverse-dep." **Subtle tension, not a conflict:** A2 §6 lists a
  *linear* bottom-up order; A5 §2/§7 proves there's a **dependency cycle** so the
  cluster must be deleted as **one atomic batch**, not strictly one-at-a-time. A5
  is more correct here (it has the `cargo metadata` evidence). **Fix:** A2 §6
  should note the cycle and say "delete the cloud cluster as one branch" rather
  than implying a strict linear sequence. Minor.
- **A5 Wave D (`graphql` → `warp_graphql_schema`)** ↔ A2 §1 defers graphql; A5
  owns it. Consistent (and resolves G3's orphan).
- **A5 Wave E (`remote_server`, gated on the `warp_files` edit §4.1)** ↔ A2 §6
  ("remote_server after warp_files arms removed, §6"). Consistent — both gate
  remote_server deletion on the warp_files edit.
- **A5 Wave F (`warp_web_event_bus`)** ↔ A2 §1/§6 (wasm-only, deletable with zero
  desktop edits). Consistent.
- **A5 Wave G (`input_classifier`/`nld`, post-AI)** ↔ A1 §7.2 / doc 05 Tier C.
  Consistent.

**Global-order cross-check vs doc 05's suggested order** (Telemetry → Code-editor
→ MCP/computer_use/voice → AI → Cloud → workflows/graphql):
- A3 (telemetry) is first and self-contained — matches.
- A4 (code-editor) §0/§5 **overrides** doc 05's "code-editor before AI" with a
  hard sequencing rule: **AI must come first or jointly**, because
  `ai/blocklist/**` imports `CodeEditorView`/`ReviewComment*`. This is a
  **deliberate, well-justified deviation from doc 05** and A4 documents it
  explicitly. A5's Wave B (AI) precedes nothing that contradicts this, but A5
  doesn't sequence the `app/`-layer code-editor work (it's crate-level only). **No
  conflict, but the execution plan must adopt A4's "AI before/with code-editor"
  rule over doc 05's literal ordering.** Recommend updating doc 05 §103's order to
  match A4, or add a note that A4 supersedes it.
- A1/A2 interleave (auth is the AI backend's auth) — A1 §7 and A2 §7 both say
  "coordinate AI + cloud." A5 Wave B (AI) then Wave C (cloud) respects this.

**Bottom line:** A5's crate-deletion order is the correct *crate-level* spine and
it agrees with every per-area order. The two adjustments are (a) A2 §6 should note
the cloud cycle is atomic (A5 is right), and (b) the **global app-layer order must
follow A4's AI-first override, not doc 05's literal code-editor-first**.

---

## 7. Verdict — Ready to execute Wave 2?

**YES — conditionally.** The spec set is coherent, complete enough, and
execution-grade. Eight specs, one substantive correctness fix (C1), three product
decisions (C3 mermaid, C4 lsp, C6 classifier — all already flagged by the specs as
judgment calls), and a handful of hygiene/ownership reconciliations (G1, G2, G3).
None of these block *starting* Wave 2; they block specific later waves. The
lowest-risk, highest-signal first moves are unambiguous and agreed across all
specs.

The dependency structure makes the start safe: **A8's "shrink `default` first" +
A3's "telemetry is already no-op in OSS" mean the first concrete steps touch only
Tarp-owned files, are individually buildable, and are reversible.**

### Recommended first 3 concrete steps for Wave 2

1. **Apply A8 §6's verbatim `default` block** (replace `app/Cargo.toml:71–260`,
   the Verified 187-entry array, with the ~50-entry terminal-only set) **and
   rewire `gui = ["voice_input"]` → `gui = []`** (`app/Cargo.toml:705`, Verified).
   Then `cargo build -p warp` and smoke-test launch. This is one Tarp-owned file
   edit, the single highest-leverage de-Warp move, and it makes every subsequent
   deletion a shrinking buildable target. Fix any non-`cfg`-gated call sites the
   build surfaces (A8 §2 step 2); specifically verify `markdown_mermaid`,
   `global_search`, `command_palette_file_search` still compile (C3 / §4 caveat).

2. **Lock in the telemetry no-op baseline (A3 Phase 1, zero tracked edits):**
   build `cargo build -p warp --features release_bundle` (default channel Oss, no
   `crash_reporting`/`cocoa_sentry`) and confirm it builds, runs, and emits
   nothing. This validates A3's central finding before any deletion. In the same
   step, **resolve C4 (lsp keep-vs-delete) with a 30-minute `rg` audit** of whether
   any non-removed terminal path consumes `crates/lsp` — the answer gates Wave 4/7
   crate decisions and reconciles A4 §7 vs A5 §5.

3. **Reconcile the spec conflicts into the execution plan before any crate is
   deleted:** (a) add `crates/persistence` to A5's blocker list as a forced
   tracked-crate edit cross-referencing A1 §5.1 (C1 — the one correctness fix);
   (b) add the feature-ownership ledger to
   `docs/09-parallel-execution-plan.md` resolving G2 (cloud_conversations,
   agent_shared_sessions, pr_comments_*, global_ai_analytics_*); (c) record the
   product decisions on `markdown_mermaid` (C3) and `input_classifier` (C6); and
   (d) adopt A4's "AI before/with code-editor" override of doc 05's order, plus
   A3↔A8 `RELEASE_FLAGS` cross-reference (§5). These are doc edits, not code, and
   they prevent the "falls between two specs" failure mode once parallel execution
   starts.

Steps 1–2 are buildable, reversible, Tarp-owned, and validate the two load-bearing
findings (shrink-default + telemetry-already-off). Step 3 closes the reconciliation
gaps so Wave 2's parallel crate work doesn't strand the contested surfaces.

---

## Appendix — claims spot-verified against the repo (commit `2bb3a04b`)

- `crates/persistence/Cargo.toml:23` `warp_multi_agent_api.workspace = true`;
  `model.rs:8-9` + `model_tests.rs:3` `use warp_multi_agent_api` — confirms C1.
- `app/Cargo.toml` `default` array = **exactly 187 entries** — confirms A8 (not
  the "~190" in doc 05 / A1).
- `Cargo.toml:11-22` `default-members` includes `crates/graphql` and
  `crates/editor` — confirms A2/A5/A6 (graphql deletion needs a default-members
  edit; editor stays).
- `app/Cargo.toml:705` `gui = ["voice_input"]` — confirms the A1/A8 rewire.
- Disputed feature names exist: `welcome_tab`, `avatar_in_tab_bar`, `projects`,
  `configurable_toolbar`, `creating_shared_sessions` all present in
  `app/Cargo.toml` — A8's REMOVE list references real features.
