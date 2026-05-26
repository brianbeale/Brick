#!/usr/bin/env bash
set -euo pipefail

# Bootstrap the Android build environment for Brick.
# Run once per machine (or after a clean clone without the wrapper jar).
# After this, `make android-debug` and `./gradlew` work without Android Studio.

ANDROID_DIR="$(cd "$(dirname "$0")/.." && pwd)"
JAR="$ANDROID_DIR/gradle/wrapper/gradle-wrapper.jar"

echo "==> Checking Android build prerequisites..."

# ── Java 17+ ──────────────────────────────────────────────────────────────────
if [ -z "${JAVA_HOME:-}" ]; then
    for candidate in \
        /usr/lib/jvm/java-17-openjdk-amd64 \
        /usr/lib/jvm/java-17-openjdk \
        /usr/lib/jvm/temurin-17 \
        /Library/Java/JavaVirtualMachines/temurin-17.jdk/Contents/Home; do
        if [ -x "$candidate/bin/java" ]; then
            export JAVA_HOME="$candidate"
            break
        fi
    done
fi

JAVA_CMD="${JAVA_HOME:+$JAVA_HOME/bin/}java"
if ! command -v "$JAVA_CMD" &>/dev/null && ! [ -x "$JAVA_CMD" ]; then
    echo "ERROR: Java not found. Install with:"
    echo "  sudo apt-get install -y openjdk-17-jdk   # Debian/Ubuntu/WSL2"
    exit 1
fi

java_ver=$("$JAVA_CMD" -version 2>&1 | awk -F '"' '/version/ {print $2}' | cut -d. -f1)
if [ "$java_ver" -lt 17 ]; then
    echo "ERROR: Java 17+ required (found Java $java_ver)."
    exit 1
fi
echo "  Java $java_ver OK  (JAVA_HOME=${JAVA_HOME:-system default})"

# ── Android SDK ───────────────────────────────────────────────────────────────
if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
    for candidate in "$HOME/Android/Sdk" "$HOME/Library/Android/sdk" "/opt/android-sdk"; do
        if [ -d "$candidate" ]; then
            export ANDROID_HOME="$candidate"
            break
        fi
    done
fi

if [ -z "${ANDROID_HOME:-}" ]; then
    echo ""
    echo "WARNING: Android SDK not found (ANDROID_HOME not set)."
    echo "  Option A — Command-line tools only:"
    echo "    1. Download: https://developer.android.com/studio#command-line-tools-only"
    echo "    2. Unzip to ~/Android/Sdk/cmdline-tools/latest/"
    echo "    3. Run: sdkmanager 'platform-tools' 'platforms;android-35' 'build-tools;35.0.0'"
    echo "    4. Add to ~/.bashrc:  export ANDROID_HOME=\$HOME/Android/Sdk"
    echo "  Option B — Android Studio: open android/, let it sync once."
    echo ""
else
    echo "  ANDROID_HOME=$ANDROID_HOME OK"
fi

# ── Gradle wrapper jar ────────────────────────────────────────────────────────
if [ -f "$JAR" ]; then
    echo "  gradle-wrapper.jar already present — skipping"
else
    mkdir -p "$ANDROID_DIR/gradle/wrapper"
    GRADLE_VERSION="8.9"
    DIST_URL="https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip"
    TMP=$(mktemp -d)

    echo "  Downloading Gradle $GRADLE_VERSION (~130 MB)..."
    curl -fsSL --progress-bar "$DIST_URL" -o "$TMP/gradle.zip"

    echo "  Extracting Gradle $GRADLE_VERSION..."
    python3 -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
        "$TMP/gradle.zip" "$TMP"

    GRADLE_BIN="$TMP/gradle-${GRADLE_VERSION}/bin/gradle"
    chmod +x "$GRADLE_BIN"

    echo "  Generating wrapper via 'gradle wrapper'..."
    GRADLE_USER_HOME="$TMP/.gradle_home" \
        "$GRADLE_BIN" wrapper \
            --gradle-version "$GRADLE_VERSION" \
            --distribution-type bin \
            --project-dir "$ANDROID_DIR" \
            -q

    rm -rf "$TMP"
    echo "  gradle-wrapper.jar ready"
fi

# ── cargo-ndk ─────────────────────────────────────────────────────────────────
if ! command -v cargo-ndk &>/dev/null; then
    echo ""
    echo "WARNING: cargo-ndk not found. Install with:"
    echo "  cargo install cargo-ndk"
    echo ""
else
    echo "  cargo-ndk $(cargo-ndk --version 2>&1 | head -1) OK"
fi

# ── Android Rust targets ──────────────────────────────────────────────────────
echo "  Checking Rust Android targets..."
for target in x86_64-linux-android aarch64-linux-android; do
    if rustup target list --installed | grep -q "$target"; then
        echo "    $target OK"
    else
        echo "    Installing $target..."
        rustup target add "$target"
    fi
done

echo ""
echo "==> Setup complete. Next steps:"
echo "    make android-avd      # create the ZFold4_dev emulator"
echo "    make android-emu      # start the emulator"
echo "    make android-install  # build Rust + Kotlin, install APK"
