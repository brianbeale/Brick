ANDROID_SDK  ?= $(HOME)/Android/Sdk
ADB          := $(ANDROID_SDK)/platform-tools/adb $(if $(DEVICE),-s $(DEVICE),)
GRADLEW      = cd android && ANDROID_HOME=$(ANDROID_SDK) ./gradlew

# ABI for the emulator (x86_64 for AVDs on an x86_64 host; use aarch64-linux-android for physical device)
ANDROID_ABI  ?= x86_64-linux-android

# ── One-time setup ────────────────────────────────────────────────────────────

android-setup:
	@bash android/scripts/setup-android.sh

android-avd:
	@bash android/scripts/setup-avd.sh

# ── Emulator ──────────────────────────────────────────────────────────────────

android-emu:
	@bash android/scripts/android-emu-start.sh

android-emu-stop:
	$(ADB) emu kill

# ── Build ─────────────────────────────────────────────────────────────────────

# Compile Rust to a .so and copy it where Gradle expects it.
android-build-rust:
	cargo ndk -t $(ANDROID_ABI) \
		-o android/demo/src/main/jniLibs \
		build --release

android-debug: android-build-rust
	$(GRADLEW) :demo:assembleDebug

# ── Deploy ────────────────────────────────────────────────────────────────────

android-install: android-build-rust
	$(GRADLEW) :demo:installDebug

android-deploy: android-install

# ── Dev tools ─────────────────────────────────────────────────────────────────

android-devices:
	$(ADB) devices

android-log:
	$(ADB) logcat -s BrickDemo:* AndroidRuntime:E

android-log-full:
	$(ADB) logcat

android-clean:
	$(GRADLEW) :demo:clean
	rm -rf android/demo/src/main/jniLibs

# ── Rust tests (host) ─────────────────────────────────────────────────────────

test:
	cargo test --lib

.PHONY: android-setup android-avd android-emu android-emu-stop \
        android-build-rust android-debug android-install android-deploy \
        android-devices android-log android-log-full android-clean test
