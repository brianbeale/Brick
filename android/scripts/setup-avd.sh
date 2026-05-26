#!/usr/bin/env bash
set -euo pipefail

# Set up the Android emulator AVD for a Z Fold 4-compatible device.
# Requires: Android SDK with cmdline-tools installed and ANDROID_HOME set.
# Run `make android-setup` first if you haven't already.

AVD_NAME="${AVD_NAME:-ZFold4_dev}"
API_LEVEL="${API_LEVEL:-34}"
ABI="x86_64"
PACKAGE="system-images;android-${API_LEVEL};google_apis_playstore;${ABI}"

# ── Resolve Android SDK ───────────────────────────────────────────────────────
if [ -z "${ANDROID_HOME:-}" ]; then
    for candidate in "$HOME/Android/Sdk" "$HOME/Library/Android/sdk" "/opt/android-sdk"; do
        [ -d "$candidate" ] && { export ANDROID_HOME="$candidate"; break; }
    done
fi
[ -z "${ANDROID_HOME:-}" ] && { echo "ERROR: ANDROID_HOME not set. Run make android-setup first."; exit 1; }

export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"

echo "==> Android SDK: $ANDROID_HOME"

# ── Install required SDK packages ─────────────────────────────────────────────
echo "==> Installing SDK packages (safe to re-run)..."
sdkmanager --install \
    "platform-tools" \
    "emulator" \
    "platforms;android-${API_LEVEL}" \
    "build-tools;35.0.0" \
    "$PACKAGE" \
    < /dev/null   # accept all licences non-interactively (already accepted at SDK install time)

# ── Locate the best hardware profile for Z Fold 4 ────────────────────────────
# The Galaxy Z Fold 4's inner display is 7.6" 2176×1812 at ~374 dpi.
# The Pixel Fold profile (7.6" 2208×1840) is the closest built-in CLI option.
# Android Studio users: Device Manager → New → search "Galaxy Z Fold4" for the
# Samsung-provided profile which includes the outer screen skin.

echo "==> Checking available foldable device profiles..."
DEVICE_ID=""
for probe in "pixel_fold" "7.6in Foldable" "7in Foldable"; do
    if avdmanager list devices 2>/dev/null | grep -q "\"$probe\""; then
        DEVICE_ID="$probe"
        echo "  Using profile: $probe"
        break
    fi
done

if [ -z "$DEVICE_ID" ]; then
    echo "  No built-in foldable profile found — using generic tablet (8in)"
    DEVICE_ID="Nexus 10"
fi

# ── Create or recreate the AVD ────────────────────────────────────────────────
if avdmanager list avd 2>/dev/null | grep -q "Name: $AVD_NAME"; then
    echo "==> AVD '$AVD_NAME' already exists — deleting and recreating..."
    avdmanager delete avd --name "$AVD_NAME"
fi

echo "==> Creating AVD: $AVD_NAME"
echo "no" | avdmanager create avd \
    --name "$AVD_NAME" \
    --package "$PACKAGE" \
    --device "$DEVICE_ID" \
    --force

# ── Patch config.ini for foldable display settings ───────────────────────────
AVD_CONFIG="$HOME/.android/avd/${AVD_NAME}.avd/config.ini"
if [ -f "$AVD_CONFIG" ]; then
    echo "==> Patching AVD config for foldable display..."
    # Z Fold 4 inner display: 7.6" 2176×1812 ~374dpi
    sed -i \
        -e 's/^hw\.lcd\.width=.*/hw.lcd.width=2176/' \
        -e 's/^hw\.lcd\.height=.*/hw.lcd.height=1812/' \
        -e 's/^hw\.lcd\.density=.*/hw.lcd.density=374/' \
        -e 's/^hw\.ramSize=.*/hw.ramSize=8192/' \
        "$AVD_CONFIG"

    # Enable hardware keyboard for easier dev typing
    if ! grep -q "^hw.keyboard=" "$AVD_CONFIG"; then
        echo "hw.keyboard=yes" >> "$AVD_CONFIG"
    fi
fi

echo ""
echo "==> AVD '$AVD_NAME' ready."
echo ""
echo "  Start emulator:  make android-emu"
echo "  Install APK:     make android-install   (works on emulator or physical device)"
echo ""
echo "  Physical device (Z Fold 4):"
echo "    1. Enable USB Debugging: Settings → About Phone → tap Build Number ×7"
echo "       then Settings → Developer Options → USB Debugging → ON"
echo "    2. Connect USB cable — accept the 'Allow USB debugging?' dialog on phone"
echo "    3. adb devices           # should list your phone"
echo "    4. make android-install  # installs directly to phone"
echo ""
echo "  Wireless (Android 11+, no cable needed after first pair):"
echo "    Settings → Developer Options → Wireless Debugging → Pair device with code"
echo "    adb pair <ip>:<pairing-port>   # enter the 6-digit code shown on phone"
echo "    adb connect <ip>:<debug-port>"
echo "    make android-install"
