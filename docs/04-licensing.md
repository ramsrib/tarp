# 04 — Licensing

Full license audit of what the repo ships and redistributes, plus the concrete
fixes needed for a clean Tarp release.

## License model (as inherited from Warp)

- **Dual license:**
  - `warpui` + `warpui_core` crates → **MIT** (`LICENSE-MIT`).
  - Everything else in the repo → **AGPL-3.0-only** (`LICENSE-AGPL`).
  - `[workspace.package] license = "AGPL-3.0-only"` is the default for member crates.
- **Copyright:** "Copyright (C) 2020-2026 Denver Technologies, Inc." (Denver
  Technologies is Warp's legal entity). Per-bundle `copyright = "© 2025, Denver
  Technologies, Inc"` in `app/Cargo.toml`.

### What AGPL means for Tarp
- AGPL is preserved and **compatible** with the fork. We must keep upstream
  copyright + license notices and offer source.
- The AGPL §13 "network use = distribution" clause is the clause that mattered for
  Warp's cloud; once cloud/server features are removed, it's largely moot — but the
  license stays AGPL regardless.
- We **add** a Tarp copyright line alongside Denver Technologies; we do **not**
  replace or remove theirs.

## Third-party attribution tooling

Two config files drive license compliance, kept in sync by
`script/check_license_config_sync` (enforced in CI):

- **`about.toml`** (cargo-about) — generates third-party license attribution.
- **`deny.toml`** (cargo-deny) — bans/advisories/license allow-list/source allow-list.

The accepted-license allow-list (must match between both files):
`0BSD, Apache-2.0, BSD-2-Clause, BSD-3-Clause, BSL-1.0, CC0-1.0,
CDLA-Permissive-2.0, ISC, MIT, MPL-2.0, Unlicense, Unicode-3.0,
Unicode-DFS-2016, Zlib`.

`deny.toml` highlights:
- `[sources] allow-org = { github = ["warpdotdev"] }` — blanket-trusts all
  warpdotdev git repos.
- `[advisories] ignore` — 9 RUSTSEC advisories accepted (unmaintained crates:
  `bincode`, `derivative`, `get-size`, `instant`, `memmap`, `paste`/`metal`,
  `safemem`, …). These carry over to Tarp; worth revisiting after dep changes.
- `[bans] multiple-versions = "allow"`, `wildcards = "deny"` (path deps exempt).

## Audit findings & required fixes

1. **`about.toml` comment is stale and wrong.** It lists 6 crates
   (`command-corrections`, `session-sharing-protocol`, `warp-command-signatures`,
   `warp-completion-metadata`, `warp-workflows`, `warp-workflows-types`) as
   "not open-sourced yet, no explicit license" and tells cargo-about to skip them.
   **They are open-sourced now** (MIT / Apache-2.0 / AGPL — see
   [`03-dependencies.md`](03-dependencies.md)).
   - **Fix:** for the 3 crates we keep, remove them from the skip list so they're
     properly attributed; re-add the `--fail` flag to the cargo-about generate
     invocation. The 2 removed crates drop out when their code goes.

2. **Trademark.** "Warp", the Warp logo, and brand assets are Warp's IP. AGPL
   covers *code*, not trademarks. Every shipped artifact must be scrubbed of Warp
   marks (see [`06-branding-and-rename.md`](06-branding-and-rename.md)). Tarp
   branding must be visibly distinct.

3. **Bundled binary assets** need a redistribution-rights check:
   - Fonts — `script/patch_font_with_warp_glyph` patches a font with a Warp glyph;
     confirm the base font's license allows redistribution and **remove the Warp
     glyph**.
   - Icons, PNGs, SVGs under `images/`, `resources/`, `app/channels/*/icon/`.
   - Syntax themes, sounds, `app/assets/bundled/`.

4. **Sentry framework** is bundled for some channels
   (`Sentry-Dynamic-WithARM64e.xcframework`). Removed with telemetry (see removal
   map); drops a redistribution concern.

5. **Regenerate attribution** after dependency changes: run cargo-about to produce
   a `THIRD_PARTY_LICENSES` file to ship with releases (Warp serves this at
   `docs.warp.dev/help/licenses`; Tarp should ship it in-repo / in-bundle).

6. **`deny.toml` source trust.** Long-term, replace
   `allow-org = ["warpdotdev"]` with an explicit allow-list (or a Tarp fork org)
   so the supply chain isn't implicitly tied to Warp's org.

## Net assessment

The licensing situation is **clean and fork-friendly** — AGPL + MIT, good tooling
already in place. The work is: (a) fix the stale `about.toml` skip-list, (b) scrub
trademarks/brand assets from shipped artifacts, (c) regenerate third-party
attribution after deps change, (d) add Tarp copyright without removing Warp's.
No license blocks the fork.

---

## Trademark (separate from the code license)

The AGPL grant is **irrevocable while we comply** (§2): the upstream copyright
holder cannot retract the license on the forked snapshot or force a takedown of
compliant AGPL code. §8 termination only triggers on a violation, and a first
violation is curable within 30 days. **The copyright axis is safe** — stay AGPL,
ship source, keep notices.

Trademark is a separate matter, which AGPL does not cover. Brand identity is
project-owned, so Tarp ships a fully distinct identity:

### Branding requirements (do before first public release)
1. **Distinct name, logo, and visual identity.** The Tarp name and the literal-tarp
   visual stand on their own; the published rationale is the tarp metaphor (simple
   shelter / stripped-back cover / practical material).
2. **Full removal of upstream brand assets** — name, logo, glyph, icons, bundle IDs
   (`dev.warp.*`), URL schemes, and any other upstream marks. See
   [`06-branding-and-rename.md`](06-branding-and-rename.md). Verify the built
   artifact, not just source.
3. **Affiliation disclaimer** in README + About: "Tarp is an independent fork and is
   not affiliated with, endorsed by, or sponsored by the upstream project."
4. **Nominative fair use only**: factual "a fork of Warp" is fine; never brand with
   an upstream mark or imply endorsement.
5. **Trademark clearance search** before first public release.

### Timing
Brand identity must be locked and the checklist complete **before first public
release / community-building**.
