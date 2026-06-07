# Releasing Tarp

Tarp releases are **tag-driven**. Pushing a semver tag builds the macOS bundle on
a GitHub-hosted runner and publishes a GitHub Release with the artifacts attached.
Cadence is stability-first: burst early, settle to roughly monthly.

## Cutting a release

```sh
# 1. Make sure main is green (CI) and you're at the commit you want to ship.
git checkout main && git pull

# 2. Tag with a semver version and push the tag.
git tag v0.1.0
git push origin v0.1.0
```

That's it. The [`Release`](.github/workflows/release.yml) workflow then:

1. Builds the OSS channel bundle for macOS arm64 (`./script/bundle --channel oss --nouniversal`).
2. Generates `THIRD_PARTY_LICENSES.txt` via `cargo-about` (bundled into the app and
   attached as a standalone asset).
3. Packages `Tarp-macos-arm64.dmg`.
4. Creates a GitHub Release for the tag with auto-generated release notes and the
   artifacts attached.

### Dry run (no Release created)

From the **Actions → Release → Run workflow** menu, leave `publish` unchecked. The
build runs and uploads the artifacts to the run (downloadable from the run page)
without creating a public Release. Use this to validate a build before tagging.

## Signing & notarization

**Releases are signed (Developer ID) and notarized.** Builds open cleanly through
Gatekeeper (`spctl` → *accepted, Notarized Developer ID*). Signing + notarization
**auto-activate** whenever these repo secrets are present (Settings → Secrets and
variables → Actions) — no workflow edits; if the cert secret is absent the build
falls back to a valid ad-hoc signature (openable with a one-time Gatekeeper step):

| Secret | What it is |
|---|---|
| `APPLE_DEVELOPER_ID_CERT` | base64 of the *Developer ID Application* certificate `.p12` |
| `APPLE_DEVELOPER_ID_CERT_PASSWORD` | password for that `.p12` |
| `APPLE_TEAM_ID` | 10-char Apple Developer Team ID |
| `APPLE_CODESIGN_KEYCHAIN_PASSWORD` | arbitrary; scoped to the throwaway CI keychain |
| `APPLE_NOTARIZATION_API_KEY` | base64 of the App Store Connect API key `.p8` |
| `APPLE_NOTARIZATION_API_KEY_ID` | the key ID (the `<id>` in `AuthKey_<id>.p8`) |
| `APPLE_NOTARIZATION_API_ISSUER` | the API key's issuer ID (a UUID) |

When present, the workflow builds with `--read-passwords-from-env`; the bundle
script signs (`codesign -o runtime --timestamp`), notarizes (`xcrun notarytool
submit --wait`), and staples the ticket. `APPLE_TEAM_ID` flows through to the
bundle script's signing identity (it falls back to the upstream value if unset).

**Notarization auth.** The bundle script prefers an [App Store Connect API
key](https://appstoreconnect.apple.com/access/integrations/api) (`.p8` + key ID +
issuer) — not tied to a personal Apple ID, survives password changes. Create one
under *Users and Access → Integrations → App Store Connect API* with the
*Developer* role; download the `.p8` once. The script still accepts the older
Apple-ID + app-specific-password pair (`WARP_NOTARIZATION_APPLE_ID` /
`WARP_NOTARIZATION_PASSWORD`) as a fallback if the API-key vars are unset.

## Installing (macOS)

Signed + notarized releases just open: open the DMG, drag **Tarp** to Applications,
launch it, and click **Open** on the standard one-time *"downloaded from the
Internet"* dialog. No "unverified" warning, no workaround.

**Fallback — an unsigned/ad-hoc build** (if a release is ever cut without the
signing secrets): the bundle still gets a valid ad-hoc signature
(`codesign --force --deep --sign -`), so macOS shows the *openable* "Apple could
not verify…" prompt (not "damaged"). Open it once via **System Settings → Privacy
& Security → Open Anyway**, or `xattr -dr com.apple.quarantine /Applications/Tarp.app`.

## Not yet automated

- **Intel / universal2 macOS** — v1 is arm64-only (`--nouniversal`). A universal
  build is a later enhancement.
- **Linux / Windows** — the bundle scripts support both (`script/linux/bundle`,
  `script/windows/bundle.ps1`); add matrix legs to `release.yml` once those targets
  are build-verified for Tarp.
- **Auto-update** — none today (`autoupdate_config: None`). Updates are manual via
  GitHub Releases. See [`docs/BACKLOG.md`](docs/BACKLOG.md).
