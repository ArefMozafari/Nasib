#!/bin/zsh
# Builds Nasib.app into build/. With "install" it also copies the app to ~/Applications and launches it;
# with "release" it runs the tests first and zips the app as build/Nasib-<version>-macos.zip.
set -euo pipefail
cd "$(dirname "$0")"

APP_NAME="Nasib"
BUILD_DIR="build"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
INSTALL_DIR="$HOME/Applications"
RELEASE_PLATFORM_SUFFIX="macos"

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
  # actool resolves relative paths against the PWD variable, not the real working directory, so pass absolute ones.
  local project_dir
  project_dir="$(pwd -P)"
  actool "$project_dir/Design/$APP_ICON_NAME.icon" --compile "$project_dir/$resources" --app-icon "$APP_ICON_NAME" \
    --include-all-app-icons --platform macosx --minimum-deployment-target "$MINIMUM_SYSTEM_VERSION" \
    --output-partial-info-plist "$project_dir/$ICON_WORK_DIR/icon.plist" >/dev/null
  local glyph
  for glyph in "${MENU_BAR_GLYPHS[@]}"; do
    render_svg "Design/$glyph.svg" "$resources/$glyph.png" "$MENU_BAR_GLYPH_POINTS"
    render_svg "Design/$glyph.svg" "$resources/$glyph@2x.png" $(( MENU_BAR_GLYPH_POINTS * 2 ))
  done
}

if [[ "${1:-}" == "release" ]]; then
  swift test
  zsh Tests/Engine/ws-find.test.zsh
fi

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

if [[ "${1:-}" == "release" ]]; then
  version=$(plutil -extract CFBundleShortVersionString raw Packaging/Info.plist)
  release_zip="$BUILD_DIR/$APP_NAME-$version-$RELEASE_PLATFORM_SUFFIX.zip"
  rm -f "$release_zip"
  # ditto keeps the bundle's signature and extended attributes intact, which a plain zip can break.
  ditto -c -k --sequesterRsrc --keepParent "$APP_BUNDLE" "$release_zip"
  echo "released $release_zip"
fi
