# 09 — Parallel Execution Plan (agent-driven)

How to drive the Tarp work with multiple agents in parallel. The organizing
principle: **what parallelizes is analysis, prep, and the peripheral surfaces
(branding/CI/docs/hygiene); the de-Warp code surgery itself is mostly sequential**
because every removal converges on shared `app/` files. So we front-load a wide
parallel wave that produces precise specs, then execute the surgery sequenced by
those specs.

## Parallelism reality

| Work | Parallelizes? | Why |
|---|---|---|
| Removal-surface mapping (AI/cloud/telemetry/editor) | ✅ Fully | Read-only; independent outputs |
| Feature-flag categorization, path-split, dep graph | ✅ Fully | Read-only analysis |
| Branding inventory | ✅ Fully | Read-only |
| CI/release plan + draft workflows | ✅ Fully | New files under `.github/`, no code overlap |
| Governance docs (README/CoC/SECURITY/NOTICE) | ✅ Fully | Root `.md` files, no code overlap |
| Upstream remote + sync scaffolding | ✅ Mostly | New files + git config |
| **De-Warp code removal (AI/cloud/accounts)** | ⚠️ **Limited** | All edit shared `app/` files (mod/lib/settings/command dispatch) → agents conflict |
| Self-contained crate *deletions* | ✅ In worktrees | Once `app/` no longer references them |
| Rebrand string/asset replacement | ✅ By area | Partition by bundle-ids / shell / UI strings / icons |

**Conclusion:** run Waves 0–1 as a broad parallel fan-out now; execute Wave 2
(surgery) as a guided sequence (with worktree-parallel crate deletions at the end).

---

## Wave 0 — Analysis & specs (parallel, read-only) ⟶ START HERE

8 independent agents, each producing a spec/worklist. No code changes, zero
conflicts. These outputs make Wave 2 mechanical.

| # | Agent task | Output |
|---|---|---|
| A1 | Map the **AI/agent** removal surface across `app/` + crates (`ai`, `mcp`, `computer_use`, `voice_input`, `warp_multi_agent_api`): every file, the feature flags, UI/settings/command surfaces, the removal order. | `docs/removal/ai.md` |
| A2 | Map the **cloud/accounts/sharing** surface (`cloud_*`, `firebase`, `warp_server_*`, `session-sharing-protocol`, `onboarding`, auth, server URLs). | `docs/removal/cloud-accounts.md` |
| A3 | Map the **telemetry/analytics/crash-reporting** surface; identify the facade(s) so they can be stubbed to no-op first. | `docs/removal/telemetry.md` |
| A4 | Map the **code-editor ("Warp Code")** feature cluster (`*_code_editor`, `tabbed_editor_view`, `file_tree`, `code_review_*`, `create_project_flow`). Disambiguate from the `editor` input-buffer crate. | `docs/removal/code-editor.md` |
| A5 | Finalize the **tracked-vs-owned path split** + `cargo tree` reverse-dep graph for every removable crate (what references what). | `docs/removal/dep-graph.md` |
| A6 | Exhaustive **branding inventory**: every Warp string / asset / bundle-id / `.desktop` / shell-integration ref, each with a proposed Tarp replacement. | `docs/removal/branding-map.md` |
| A7 | **CI/release** plan: per-workflow keep/cut/replace + draft a slim PR CI and a tag-driven release workflow (don't enable yet). | `docs/removal/ci-plan.md` + draft yml |
| A8 | **Feature-flag** categorization: all 292 features → keep / remove / default-change; define the **minimal terminal-only `default` set**. | `docs/removal/feature-flags.md` |

> Run A1–A8 concurrently. Each reads the repo + the existing `docs/` audit and
> writes one spec file. A `code-review`/critic agent can then sanity-check the set.

## Wave 1 — Hygiene & scaffolding (parallel; can overlap Wave 0)

Independent of code surgery — touches docs, `.github/`, git config only.

| # | Agent task | Depends on |
|---|---|---|
| H1 | Rewrite `README.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CONTRIBUTING.md`, issue/PR templates for Tarp; add `NOTICE` + affiliation disclaimer. | — |
| H2 | Prune Warp-internal CI workflows; land the slim CI from A7. | A7 |
| H3 | Add `upstream` remote, `upstream-main` mirror, `UPSTREAM_SYNC` file, and a `script/sync-upstream` skeleton. | A5 |
| H4 | Branch rename `master → main` (GitHub setting + ref updates). | ⚠️ needs `gh`/owner action |

## Wave 2 — De-Warp surgery (guided sequence, NOT free parallel)

Driven by Wave-0 specs. Rebuild + run M1 smoke test (`./script/run --dont-open`)
after each step. Order = the de-risked sequence from
[`05-removal-map.md`](05-removal-map.md):

1. **Telemetry** → stub the facade to no-op (A3). Smallest, most isolating.
2. **Code-editor cluster** → flag-off + delete (A4).
3. **MCP + computer_use + voice_input** (A1).
4. **AI core** (`ai`, `warp_multi_agent_api`, `agent_*` features) (A1) — the big one.
5. **Cloud + sharing + accounts** (`cloud_*`, `session-sharing`, auth, onboarding,
   firebase, server URLs) (A2).
6. **Untangle/re-home `workflows`**; drop now-dead crates.
7. **Prune the 2 removed git deps**; regenerate `Cargo.lock`.

> These steps are sequential because they share `app/` files. **Within** a step,
> self-contained crate *deletions* (no remaining `app/` refs) can be farmed to
> worktree-isolated agents. Apply the WS2 rule: minimize edits to
> tracked-from-upstream crates.

## Wave 3 — Rebrand (parallel by area, after Wave 2)

From A6's branding map: partition into independent agents — (a) bundle IDs/plists/
`.desktop`, (b) shell-integration scripts, (c) UI/about/settings strings, (d) icons/
logo/visual identity. Low overlap → parallel in worktrees.

## Wave 4 — Release engineering (after stripped + branded)

Land the tag-driven release workflow (A7 draft), wire packaging
(`script/{macos,linux,windows}/bundle*`), decide signing/notarization, ship the
regenerated `THIRD_PARTY_LICENSES`. Cadence per
[`07-ci-and-release.md`](07-ci-and-release.md): burst early → ~monthly once stable.

---

## Dependency graph (waves)

```
Wave 0 (A1..A8)  ──┬─▶ Wave 2 (sequenced surgery) ──▶ Wave 3 (rebrand ∥) ──▶ Wave 4 (release)
                   │
Wave 1 (H1..H4) ∥──┘   (H2←A7, H3←A5; H1,H4 independent)
```

## Product decisions (locked 2026-06-05)

Resolves the critic's open product calls (C3/C4/C6) and the autosuggestion question.

| # | Decision | Consequence |
|---|---|---|
| D1 | **Keep Mermaid** rendering in block output | `markdown_mermaid` stays in `default`; keep the `mermaid-to-svg` git dep. Resolves C3 (A8 wins). |
| D2 | **Remove NL-to-command** (type English → suggested command) | Remove `input_classifier` + `natural_language_detection`; remove the NL routing. Resolves C6. |
| D3 | **Remove LSP** (powered the code editor) | Delete `crates/lsp` (+ `node_runtime`/`warp_js` if unused) — **gated on an `rg` audit** confirming no surviving *terminal* path consumes `lsp`. Resolves C4 (A4 wins, pending verify). |
| D4 | **Keep autosuggestions/autocomplete** (terminal core) | Keep `warp_completer`, `warp_search_core` (history/workflows/files sources), `command-signatures`, `command-corrections`, and the autosuggestion UX flags. **Remove only the AI predictor layer**: `command_predictor`, `app/src/ai/predict/**`, `prompt_suggestions_via_maa`, the `ai_history:` filter. |

> **Architecture note (D4):** Warp's input assistance is a layered data-source
> mixer. The inline autosuggestion (history), the completion menu (`warp_completer`
> + specs), workflows, and corrections are all non-AI terminal sources that stay.
> `app/src/ai/predict/next_command_model.rs` builds on `warp_completer` and adds an
> AI request — removing it removes one optional source; the engine beneath is
> untouched. So Tarp keeps the good autocomplete and loses only the ML next-command
> prediction.

## Recommended immediate action

**Launch Wave 0 now** as an 8-agent parallel fan-out (+ a critic pass), and run
Wave 1 hygiene alongside it. That converts the remaining unknowns into precise
worklists and lets the harder, sequential Wave 2 proceed mechanically.
