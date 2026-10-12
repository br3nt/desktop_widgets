#!/bin/sh
# Build a release Desktop Widgets.app (menu-bar only, ad-hoc signed) into ~/Applications,
# so it can be launched from Spotlight. Usage: host/scripts/bundle-mac.sh [dest-dir]
set -eu

HOST_DIR=$(cd "$(dirname "$0")/.." && pwd)
DEST=${1:-"$HOME/Applications"}
APP="$DEST/Desktop Widgets.app"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$HOST_DIR/Cargo.toml" | head -1)

cargo build --release --manifest-path "$HOST_DIR/Cargo.toml"

# Quit a running copy so the bundle can be replaced.
pkill -x desktop-widgets 2>/dev/null || true

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$HOST_DIR/target/release/desktop-widgets" "$APP/Contents/MacOS/desktop-widgets"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Desktop Widgets</string>
  <key>CFBundleDisplayName</key><string>Desktop Widgets</string>
  <key>CFBundleIdentifier</key><string>com.br3nt.desktop-widgets</string>
  <key>CFBundleExecutable</key><string>desktop-widgets</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSAppTransportSecurity</key>
  <dict><key>NSAllowsLocalNetworking</key><true/></dict>
</dict>
</plist>
PLIST

codesign --force --sign - "$APP"
echo "Built $APP"
echo "Launch: open \"$APP\"   (or Spotlight: Desktop Widgets)"
