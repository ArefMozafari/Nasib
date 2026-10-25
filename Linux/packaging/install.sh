#!/bin/sh
# Installs Nasib from the tarball for the current user, into ~/.local, with no root needed.
#   ./install.sh            install or update
#   ./install.sh uninstall  remove it again (your history in ~/.local/share/nasib stays)
set -eu
cd "$(dirname "$0")"

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
APP_DIRECTORY="$HOME/.local/lib/nasib"
LAUNCHER="$HOME/.local/bin/nasib"
DESKTOP_NAME="com.arefmozafari.nasib.desktop"
DESKTOP_FILE="$DATA_HOME/applications/$DESKTOP_NAME"
AUTOSTART_FILE="$CONFIG_HOME/autostart/nasib.desktop"
ICON_DIRECTORY="$DATA_HOME/icons/hicolor/scalable"

if [ "${1:-}" = "uninstall" ]; then
  rm -rf "$APP_DIRECTORY"
  rm -f "$LAUNCHER" "$DESKTOP_FILE" "$AUTOSTART_FILE" "$ICON_DIRECTORY/apps/nasib.svg" \
    "$ICON_DIRECTORY/status/nasib-connected.svg" "$ICON_DIRECTORY/status/nasib-down.svg" \
    "$ICON_DIRECTORY/status/nasib-searching.svg"
  echo "Nasib is removed."
  exit 0
fi

mkdir -p "$APP_DIRECTORY" "$(dirname "$LAUNCHER")" "$(dirname "$DESKTOP_FILE")" "$ICON_DIRECTORY/apps" "$ICON_DIRECTORY/status"
install -m 755 nasib nasib-engine "$APP_DIRECTORY/"
ln -sf "$APP_DIRECTORY/nasib" "$LAUNCHER"
cp icons/apps/nasib.svg "$ICON_DIRECTORY/apps/"
cp icons/status/*.svg "$ICON_DIRECTORY/status/"
# The menu entry names the app by its full, quoted path, since ~/.local/bin may not be on the session's
# PATH and the home folder may hold a space.
sed "s|^Exec=nasib|Exec=\"$APP_DIRECTORY/nasib\"|" "$DESKTOP_NAME" > "$DESKTOP_FILE"
echo "Nasib is installed. Open it from your applications menu."
