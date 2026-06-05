# Tarp — Decision Log

Canonical record of the consequential decisions on the fork and *why*, so future
work (and future maintainers) don't relitigate settled questions. ADR-style,
newest at the bottom. Detail lives in the linked docs; this is the index + rationale.

---

## ADR-001 — Keep the name "Tarp"; brand fully distinct
**Decision:** Ship as "Tarp" with a distinct logo/visual identity; the published
rationale is the tarp metaphor only.
**Why:** Memorable, short. The copyright axis is safe (AGPL irrevocable while we
comply); residual risk is trademark, mitigated by full de-branding, an affiliation
disclaimer, nominative-fair-use ("a fork of Warp") only, and a clearance search
before first release. Lock the brand before first public release.
**Detail:** [`04-licensing.md`](04-licensing.md#trademark-separate-from-the-code-license).

## ADR-002 — Stay open-source under AGPL-3.0; do not attempt to relicense
**Decision:** Keep the inherited AGPL-3.0 (+ MIT for `warpui*`); add a Tarp
copyright alongside the upstream one; never remove upstream notices.
**Why:** We are licensees, not the copyright holder — we cannot relicense Warp's
code. AGPL is irrevocable while we comply, so they cannot force a takedown of
compliant code. Removing cloud/network features makes the §13 network clause moot
for a plain local terminal.
**Detail:** [`04-licensing.md`](04-licensing.md).

## ADR-003 — Selective upstream sync; concentrate divergence
**Decision:** Pull upstream fixes by **cherry-pick / path-scoped** of terminal-core
paths only — never a full merge. Maintain a tracked-vs-owned path split; keep
terminal-core crates close to upstream. `master` is the pristine upstream mirror.
**Why:** Upstream runs ~14 commits/day, mostly AI/cloud (irrelevant to us). Keeping
divergence concentrated in the app/AI/cloud layer keeps the worthwhile fixes
low-conflict. Don't depend on upstream — treat sync as a bonus.
**Detail:** [`08-upstream-sync.md`](08-upstream-sync.md).

## ADR-004 — Product scope of features (D1–D4)
**Decisions:** D1 keep Mermaid rendering · D2 remove NL-to-command
(`input_classifier`/`natural_language_detection`) · D3 remove LSP (powered the code
editor; verified no terminal path uses it) · D4 keep the autosuggestion/autocomplete
engine, remove only the AI predictor layer (`command_predictor`,
`app/src/ai/predict/**`, `prompt_suggestions_via_maa`).
**Why:** Keep genuinely-terminal features; drop AI-adjacent ones. The autocomplete
engine (history/completions/specs/corrections) is non-AI and core.
**Detail:** [`09-parallel-execution-plan.md`](09-parallel-execution-plan.md#product-decisions-locked-2026-06-05).

## ADR-005 — De-Warp by DISABLE first, not DELETE; ship an early release on the disabled base
**Decision (2026-06-05):** For the first release, achieve the no-AI/no-cloud
terminal by **disabling** those surfaces (the minimal default feature set, OSS
channel) rather than deleting their source. Pivot now to **branding (M3) + first
release (M4)** on the current `dewarp` base. Treat full source deletion of AI/cloud
as a **separate, optional, later project**, pursued only if binary-size / source
purity justifies the cost.

**Why:**
- **The disabled state already meets the user-facing goal.** The current `dewarp`
  build is a working terminal with AI/cloud/accounts/editor features off (cfg-gated;
  OSS dials no network, runs logged-out). "Delete" only adds smaller binary +
  source purity — not a different user experience.
- **Source deletion is disproportionately expensive and risky.** Investigation
  showed no incremental/clean path: `app/src/ai/` is 459 files / ~222k LOC, imported
  by **334 external files**; `AISettings` itself depends on it; AI is co-entangled
  with cloud/server/auth. It's one ~600–700-file coordinated AI+cloud surgery that
  can't be kept green partway through.
- **Deletion fights our own sync strategy (ADR-003).** Full deletion maximizes
  divergence from upstream, making the selective cherry-pick of terminal fixes much
  harder. Disable = low divergence = sustainable.
- **Matches the release philosophy.** A publishable Tarp now (burst of releases
  early → stabilize) beats a months-long deletion before anything ships.

**Consequences:**
- v1 ships with AI/cloud code present-but-inert (larger binary; no AI/cloud UX).
- The `voice_input` crate is the one piece already deleted (it was a clean leaf).
- Future deletion, if pursued, is one dedicated AI+cloud branch effort.
- Anything removed remains restorable via [`REMOVED.md`](REMOVED.md) + `git revert`.

**Detail:** [`removal/ai-removal-feasibility.md`](removal/ai-removal-feasibility.md).

**Supersedes:** the earlier implicit assumption (Wave 0 / `05-removal-map.md`) that
de-Warp meant deleting the AI/cloud crates up front. Those removal specs remain
valid as the blueprint *if/when* full deletion is undertaken.
