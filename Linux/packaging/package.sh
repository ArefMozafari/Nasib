#!/bin/sh
# Builds both binaries for release and writes the .deb, .rpm and .tar.gz into Linux/build/. Run it from the
# repository root inside the build container; `Linux/build.sh release` does that.
set -eu

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Linux/Cargo.toml | head -1)
PACKAGE_NAME="Nasib-$VERSION-linux-x64"
OUTPUT=Linux/build
STAGE=$(mktemp -d)
TARGET="$CARGO_TARGET_DIR/release"

cargo build --release --locked --manifest-path Engine/Cargo.toml
cargo build --release --locked --manifest-path Linux/Cargo.toml

DESKTOP_FILE="com.arefmozafari.nasib.desktop"

# Tray glyphs: the shared black glyphs in a mid gray that reads on light and dark panels alike. They stay
# plain icons, since hosts recolor symbolic ones by filling their paths, which spoils stroke-only waves.
tray_glyphs() {
  destination="$1"
  mkdir -p "$destination"
  for state in Connected Down Searching; do
    name=$(echo "$state" | tr '[:upper:]' '[:lower:]')
    sed 's/#000000/#bebebe/g' "Design/MenuBar$state.svg" > "$destination/nasib-$name.svg"
  done
}

# The tree every package installs.
install -D -m 755 "$TARGET/nasib" "$STAGE/usr/lib/nasib/nasib"
install -D -m 755 "$TARGET/nasib-engine" "$STAGE/usr/lib/nasib/nasib-engine"
mkdir -p "$STAGE/usr/bin"
ln -s ../lib/nasib/nasib "$STAGE/usr/bin/nasib"
# Named after the application ID, so Wayland shells match the window to its launcher and icon.
install -D -m 644 "Linux/packaging/$DESKTOP_FILE" "$STAGE/usr/share/applications/$DESKTOP_FILE"
install -D -m 644 Design/AppIcon.svg "$STAGE/usr/share/icons/hicolor/scalable/apps/nasib.svg"
tray_glyphs "$STAGE/usr/share/icons/hicolor/scalable/status"

rm -rf "$OUTPUT" && mkdir -p "$OUTPUT"

# .deb
DEB_ROOT=$(mktemp -d)
cp -a "$STAGE/." "$DEB_ROOT/"
# mktemp makes the folder private; the package's root entry must not carry that.
chmod 755 "$DEB_ROOT"
mkdir -p "$DEB_ROOT/DEBIAN"
sed "s/@VERSION@/$VERSION/" Linux/packaging/debian-control > "$DEB_ROOT/DEBIAN/control"
dpkg-deb --root-owner-group --build "$DEB_ROOT" "$OUTPUT/$PACKAGE_NAME.deb"

# .rpm
RPM_TOP=$(mktemp -d)
rpmbuild -bb --quiet \
  --define "_topdir $RPM_TOP" \
  --define "nasib_version $VERSION" \
  --define "nasib_stage $STAGE" \
  Linux/packaging/nasib.spec
cp "$RPM_TOP"/RPMS/x86_64/*.rpm "$OUTPUT/$PACKAGE_NAME.rpm"

# .tar.gz
TAR_ROOT=$(mktemp -d)/nasib
mkdir -p "$TAR_ROOT/icons/apps"
cp "$TARGET/nasib" "$TARGET/nasib-engine" "Linux/packaging/$DESKTOP_FILE" Linux/packaging/install.sh "$TAR_ROOT/"
cp Design/AppIcon.svg "$TAR_ROOT/icons/apps/nasib.svg"
tray_glyphs "$TAR_ROOT/icons/status"
tar -czf "$OUTPUT/$PACKAGE_NAME.tar.gz" -C "$(dirname "$TAR_ROOT")" nasib

ls -l "$OUTPUT"
