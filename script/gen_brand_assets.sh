#!/usr/bin/env bash
#
# Regenerate all Tarp brand assets from a single source logo.
#
# When you tweak the logo, replace the SOURCE file with a SQUARE, TRANSPARENT,
# rounded PNG (high-res, e.g. 1024+), then run this script. It rebuilds:
#   - the macOS/Windows app icon (padded, macOS-grid centered)
#   - the in-app About logo + the README logo (full-bleed)
#   - the GitHub social-preview banner (1280x640)
#
# Requires ImageMagick v7 (`magick`). On macOS: `brew install imagemagick`.
# After running, re-bundle the app (`./script/run`) so the new .icns is baked in.
#
# Usage:
#   script/gen_brand_assets.sh [SOURCE_LOGO_PNG]
#
set -euo pipefail

REPO_ROOT="$(unset CDPATH; cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" >/dev/null

# ----- configuration -------------------------------------------------------
SOURCE="${1:-app/channels/oss/icon/AppIcon-source.png}"   # square, transparent, rounded
TITLE="Tarp"
TAGLINE="A plain, modern terminal"
PILLARS="No AI  ·  No cloud  ·  No tracking"
BG="#0d0e0b"            # banner background (near-black olive)
TAGLINE_COLOR="#b9b9b3" # muted grey
ACCENT_COLOR="#8bbf3a"  # prompt green
# Fonts (macOS). Override if missing.
TITLE_FONT="${TITLE_FONT:-/System/Library/Fonts/Supplemental/Arial Bold.ttf}"
BODY_FONT="${BODY_FONT:-/System/Library/Fonts/Supplemental/Arial.ttf}"
# Output paths
APP_ICON_PNG="app/channels/oss/icon/no-padding/512x512.png"
APP_ICON_ICO="app/channels/oss/icon/no-padding/icon.ico"
ABOUT_LOGO="app/assets/bundled/svg/tarp-logo.png"
README_LOGO="docs/assets/tarp-logo.png"
SOCIAL_BANNER="docs/assets/tarp-social-preview.png"
# ---------------------------------------------------------------------------

command -v magick >/dev/null 2>&1 || { echo "error: ImageMagick 'magick' not found (brew install imagemagick)"; exit 1; }
[ -f "$SOURCE" ] || { echo "error: source logo not found: $SOURCE"; exit 1; }
[ -f "$TITLE_FONT" ] || { echo "error: TITLE_FONT not found: $TITLE_FONT (set TITLE_FONT=...)"; exit 1; }
[ -f "$BODY_FONT" ]  || { echo "error: BODY_FONT not found: $BODY_FONT (set BODY_FONT=...)"; exit 1; }

echo "Source logo: $SOURCE"
mkdir -p "$(dirname "$ABOUT_LOGO")" "$(dirname "$README_LOGO")" "$(dirname "$SOCIAL_BANNER")"

# 1) App icon — padded to the macOS grid (~84% art, centered, transparent margin).
echo "→ app icon (padded): $APP_ICON_PNG + $APP_ICON_ICO"
PADDED="$(mktemp -t tarp_icon_padded).png"
magick "$SOURCE" -resize 860x860 -filter Lanczos -background none -gravity center -extent 1024x1024 "$PADDED"
magick "$PADDED" -resize 512x512 "$APP_ICON_PNG"
magick "$PADDED" -define icon:auto-resize=256,128,64,48,32,16 "$APP_ICON_ICO"

# 2) Full-bleed logo for the About page + README.
echo "→ about logo:  $ABOUT_LOGO (256)"
magick "$SOURCE" -resize 256x256 -filter Lanczos "$ABOUT_LOGO"
echo "→ readme logo: $README_LOGO (200)"
magick "$SOURCE" -resize 200x200 -filter Lanczos "$README_LOGO"

# 3) GitHub social-preview banner (1280x640): logo left, title + tagline + pillars.
echo "→ social banner: $SOCIAL_BANNER (1280x640)"
magick -size 1280x640 "xc:$BG" \
  \( "$SOURCE" -resize 340x340 \) -gravity West -geometry +150+0 -composite \
  -gravity West \
  -font "$TITLE_FONT" -fill '#ffffff'        -pointsize 150 -annotate +560-40 "$TITLE" \
  -font "$BODY_FONT"  -fill "$TAGLINE_COLOR"  -pointsize 48  -annotate +566+62 "$TAGLINE" \
  -font "$BODY_FONT"  -fill "$ACCENT_COLOR"   -pointsize 32  -annotate +568+128 "$PILLARS" \
  "$SOCIAL_BANNER"

rm -f "$PADDED"
echo
echo "Done. Regenerated brand assets from $SOURCE."
echo "Next: re-bundle the app so the new .icns is baked in:  ./script/run --dont-open"
echo "      (GitHub social preview is a manual upload: Settings → Social preview → $SOCIAL_BANNER)"
