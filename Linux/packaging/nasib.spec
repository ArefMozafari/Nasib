# Packs the tree that package.sh staged; it builds nothing itself.
Name:           nasib
Version:        %{nasib_version}
Release:        1
Summary:        Keeps Windscribe connected from networks that block most of it
License:        MIT
URL:            https://github.com/ArefMozafari/Nasib
Requires:       gtk4 >= 4.6

%description
Nasib finds a Windscribe server and protocol that still work, reconnects when the tunnel drops, and
records every attempt. It needs the Windscribe app for Linux.

%install
cp -a %{nasib_stage}/. %{buildroot}/

%files
/usr/bin/nasib
/usr/lib/nasib
/usr/share/applications/com.arefmozafari.nasib.desktop
/usr/share/icons/hicolor/scalable/apps/nasib.svg
/usr/share/icons/hicolor/scalable/status/nasib-connected.svg
/usr/share/icons/hicolor/scalable/status/nasib-down.svg
/usr/share/icons/hicolor/scalable/status/nasib-searching.svg
