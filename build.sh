#!/bin/zsh
# Builds Nasib.app into build/ and, with "install", copies it to ~/Applications and launches it.
set -euo pipefail
cd "$(dirname "$0")"

APP_NAME="Nasib"
BUILD_DIR="build"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
INSTALL_DIR="$HOME/Applications"

ICON_WORK_DIR="$BUILD_DIR/icons"
APP_ICON_NAME="AppIcon"
MINIMUM_SYSTEM_VERSION="15.0"
MENU_BAR_GLYPHS=(MenuBarConnected MenuBarSearching MenuBarDown)
# Menu bar glyphs are 18 points; both scale factors ship so NSImage(named:) picks the right one.
MENU_BAR_GLYPH_POINTS=18

render_svg() {
  swift Tools/render-svg.swift "$1" "$2" "$3"
}

# Compiles the Icon Composer document into Assets.car (Liquid Glass on macOS 26+) plus an .icns
# fallback for older systems, and renders the menu bar glyph PNG pairs, all into Resources.
render_icons() {
  local resources="$APP_BUNDLE/Contents/Resources"
  rm -rf "$ICON_WORK_DIR"
  mkdir -p "$ICON_WORK_DIR"
  actool "Design/$APP_ICON_NAME.icon" --compile "$resources" --app-icon "$APP_ICON_NAME" \
    --include-all-app-icons --platform macosx --minimum-deployment-target "$MINIMUM_SYSTEM_VERSION" \
    --output-partial-info-plist "$ICON_WORK_DIR/icon.plist" >/dev/null
  local glyph
  for glyph in "${MENU_BAR_GLYPHS[@]}"; do
    render_svg "Design/$glyph.svg" "$resources/$glyph.png" "$MENU_BAR_GLYPH_POINTS"
    render_svg "Design/$glyph.svg" "$resources/$glyph@2x.png" $(( MENU_BAR_GLYPH_POINTS * 2 ))
  done
}

swift build -c release

rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
cp ".build/release/$APP_NAME" "$APP_BUNDLE/Contents/MacOS/"
cp Packaging/Info.plist "$APP_BUNDLE/Contents/"
cp Engine/ws-find.sh "$APP_BUNDLE/Contents/Resources/"
render_icons
codesign --force --sign - "$APP_BUNDLE"
echo "built $APP_BUNDLE"

if [[ "${1:-}" == "install" ]]; then
  mkdir -p "$INSTALL_DIR"
  pkill -x "$APP_NAME" 2>/dev/null || true
  rm -rf "$INSTALL_DIR/$APP_NAME.app"
  cp -R "$APP_BUNDLE" "$INSTALL_DIR/"
  open "$INSTALL_DIR/$APP_NAME.app"
  echo "installed and launched $INSTALL_DIR/$APP_NAME.app"
fi
