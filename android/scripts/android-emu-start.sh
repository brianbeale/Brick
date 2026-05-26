#!/usr/bin/env bash
# Start the ZFold4_dev emulator and wait until it boots.
#
# Requires: KVM enabled in WSL2, WSLg for display, AVD created via make android-avd.
# Override: ANDROID_SDK

set -euo pipefail

ANDROID_SDK="${ANDROID_SDK:-$HOME/Android/Sdk}"
EMULATOR="$ANDROID_SDK/emulator/emulator"

echo "==> Starting emulator…"
DISPLAY="${DISPLAY:-:0}" nice -n 15 "$EMULATOR" -avd ZFold4_dev \
    -accel on -gpu swiftshader_indirect \
    -no-boot-anim -no-snapshot-save -no-audio \
    > /tmp/brick-emu.log 2>&1 &

echo "==> Waiting for device to finish booting (may take 60–90s)…"
adb wait-for-device shell 'while [ "$(getprop sys.boot_completed)" != "1" ]; do sleep 2; done'

echo "==> Emulator ready."
echo "    Run: make android-install"
