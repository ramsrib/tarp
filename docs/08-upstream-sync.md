# 08 — Upstream Sync Strategy

How Tarp can selectively pull fixes/updates/features from upstream Warp without
inheriting the AI/cloud/account churn it exists to remove.

## The core tension

There is a direct tradeoff between **how clean the fork is** and **how easy it is
to sync upstream**:

- **Delete upstream code** (rip out AI/cloud) → maximal divergence → upstream
  merges conflict everywhere.
- **Disable upstream code** (keep files, flip features off) → minimal divergence →
  upstream merges stay clean, but the "slop" is still in the source/binary.

Tarp wants the clean fork, so it *will* diverge. The job is to **structure that
divergence so the things worth syncing stay syncable.**

## Reality check (why "just merge upstream" won't work)

Measured on this repo (2026-06-05):

- **~200 commits in ~14 days → ~14 commits/day.** PR numbers already at #12261.
- Sampling recent commits, the **majority** are in the layer Tarp removes —
  AI/agents, MCP, oz/shared-sessions, orchestration, cloud repo-indexing,
  project skills/rules. Only a **minority** are terminal-core (e.g. "Report focus
  events for normal-screen terminal apps", "Register non-recursive watchers for
  Linux", "Fix block navigation (up/down arrow keys)", "Upgrade tink-rust").

**This asymmetry is the good news:** most upstream churn is irrelevant to a plain
terminal. If Tarp concentrates its divergence in the app/AI/cloud layer and keeps
the terminal-core crates close to upstream, then **the commits worth taking are
exactly the ones least likely to conflict.**

## The decision that determines everything: the path split

Classify every path as **Tracked-from-upstream** or **Tarp-owned**, and let that
split drive *how* the de-Warp is done.

### Tracked-from-upstream (keep close to upstream; sync regularly)
The terminal essentials — fixes here are high-value and low-conflict:
- `crates/warpui`, `warpui_core`, `warpui_extras`, `ui_components` (renderer)
- `crates/warp_terminal`, `warp_core`, `command`, `editor` (input buffer)
- `crates/warp_completer` (+ `command-signatures`), `command-corrections`
- `crates/vim`, `syntax_tree`, `languages`, `markdown_parser`, `fuzzy_match`
- `crates/sum_tree`, `string-offset`, `warp_util`, `settings*`
- Shell integration (`app/assets/bundled/bootstrap/*`)
- Dependency bumps + security patches (`Cargo.toml`/`Cargo.lock`)
- The forked deps (`vte`, `winit`, `font-kit`, …) — track their upstreams too

### Tarp-owned (diverged; do NOT sync wholesale)
The layer Tarp rewrites/strips — accept divergence here:
- `app/src/**` (the 2,093-file shell where AI/cloud/account UI lives)
- `crates/ai`, `computer_use`, `voice_input`, `mcp`, `onboarding`
- `crates/cloud_*`, `firebase`, `warp_server_*`, `managed_secrets*`,
  `remote_server`, `serve-wasm`
- `Cargo.toml` feature lists / `default` set

> **Implication for M2 (de-Warp):** do removals **in the `app/` layer and via
> feature flags**, and **keep the tracked-from-upstream crates as structurally
> close to upstream as possible** (avoid gratuitous refactors/renames in them).
> Every unnecessary edit to a tracked crate is a future merge conflict. This is
> the single most important maintainability rule.

## Git mechanics

### 1. Add upstream as a remote, mirror it on a pristine branch
```sh
git remote add upstream https://github.com/warpdotdev/warp.git
git fetch upstream
# Optional: keep a never-modified mirror branch for diffing/cherry-picking
git branch upstream-main upstream/main
```
Record the last-synced upstream SHA in a tracked file, e.g. `UPSTREAM_SYNC`:
```
upstream: warpdotdev/warp
last_synced_sha: <sha>
last_synced_date: YYYY-MM-DD
```

### 2. Primary model — selective cherry-pick (recommended)
Periodically list upstream commits **touching tracked paths only** since the last
sync, then cherry-pick the relevant ones:
```sh
git fetch upstream
git log --oneline <last_synced_sha>..upstream/main -- \
  crates/warpui crates/warpui_core crates/warp_terminal crates/warp_core \
  crates/editor crates/command crates/warp_completer crates/vim \
  crates/syntax_tree crates/markdown_parser app/assets/bundled/bootstrap
# review, then:
git cherry-pick <sha1> <sha2> ...
```
- Works cleanly when the fix is in a kept crate that's close to upstream.
- Conflicts only when a fix straddles a tracked + diverged file — resolve by hand.

### 3. Path-scoped pull (for whole-file/subsystem updates)
When you want to take an entire file/dir to upstream's version (e.g. a renderer or
dep bump):
```sh
git checkout upstream/main -- crates/warpui/src/...   # then review + commit
```

### 4. What NOT to do
- **No continuous `git merge upstream/main`** into `main` — it drags the entire
  AI/cloud layer back and conflicts with every deletion.
- **No `git rebase` of the whole fork** onto upstream once divergence is large.

## Tooling & cadence

- **`script/sync-upstream`** (to build): fetch upstream, print the
  tracked-path commit list since `last_synced_sha`, classify by changelog
  category, and let you pick. Update `UPSTREAM_SYNC` on completion.
- **Changelog-driven selection:** upstream marks PRs with CHANGELOG categories
  (there's a `changelog-draft` workflow/skill). Use those markers to triage which
  commits are user-facing terminal fixes worth taking.
- **Cadence:** monthly batch sync for fixes/features; **security/dep CVEs on a
  fast-track** (watch RUSTSEC + the forked deps).
- **Always rebuild + run tests after a sync** (`./script/presubmit`); the M1
  build/bundle/run check is the smoke test.

## What to sync vs never sync

| Take from upstream | Never take |
|---|---|
| Terminal rendering / `warpui` fixes | AI / agent / orchestration features |
| Escape-sequence / `vte` parser fixes | Cloud objects, drive, repo indexing |
| Completions / command-corrections | MCP, computer-use, voice |
| Shell integration fixes | Accounts / auth / onboarding / firebase |
| Keybindings, selection, block nav | Shared sessions / oz / collaboration |
| Performance, platform/OS-version fixes | Telemetry / crash-reporting wiring |
| Dependency + **security** bumps | Code-editor / "Warp Code" cluster |

## Constraints & risks

- **Selective sync only works while upstream keeps publishing AGPL source.** They
  could close future versions or stop the OSS repo (see
  [`04-licensing.md`](04-licensing.md)). Plan for the fork to be self-sustaining;
  treat upstream sync as a bonus, not a dependency.
- **Divergence grows over time** — the more `app/` is rewritten, the more even
  terminal-core fixes may need manual adaptation. Keeping tracked crates
  upstream-shaped slows this decay.
- **Forked git deps** (`vte`, `winit`, …) have their own upstreams to track;
  vendoring + pinning protects builds if warpdotdev pulls them.

## Recommendation (summary)

1. **Define the path split now** (tracked vs owned) and let it govern the de-Warp:
   strip in the `app/` layer + via feature flags; keep terminal-core crates close
   to upstream.
2. **Add `upstream` remote + an `upstream-main` mirror; track `last_synced_sha`.**
3. **Sync by cherry-pick / path-scoped pull of tracked paths only** — never a full
   merge. Drive selection from the changelog.
4. **Monthly cadence + security fast-track; test after every sync.**
5. **Don't depend on upstream** — Tarp must stand alone if the OSS source stops.
