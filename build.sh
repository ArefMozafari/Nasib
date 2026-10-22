#!/bin/zsh
# Builds Nasib.app into build/ and, with "install", copies it to ~/Applications and launches it.
set -euo pipefail
cd "$(dirname "$0")"

APP_NAME="Nasib"
BUILD_DIR="build"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
INSTALL_DIR="$HOME/Applications"

swift build -c release

rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
cp ".build/release/$APP_NAME" "$APP_BUNDLE/Contents/MacOS/"
cp Packaging/Info.plist "$APP_BUNDLE/Contents/"
cp Engine/ws-find.sh "$APP_BUNDLE/Contents/Resources/"
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
