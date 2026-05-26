# Brick Android — Phase 12 Spike

Counter demo proving `Signal<i32>` → drain queue → Compose `Text` recomposition.

## First-time setup

```sh
make android-setup   # checks Java 17+, Android SDK, cargo-ndk; downloads Gradle wrapper
make android-avd     # creates ZFold4_dev emulator AVD (same as Ariel)
```

## Run

```sh
make android-emu     # start the emulator (WSLg for display, KVM for acceleration)
make android-install # cargo-ndk build → libbrick.so + Gradle assembleDebug + adb install
```

`android-install` is the one-shot command. It:
1. Runs `cargo ndk -t x86_64-linux-android` to produce `libbrick.so`
2. Runs `./gradlew :demo:installDebug` to build and install the APK

## Override ABI

```sh
# Physical device (arm64)
make android-install ANDROID_ABI=aarch64-linux-android
```

## Useful targets

| Target | What it does |
|---|---|
| `make android-setup` | Java / SDK / cargo-ndk / Rust targets check |
| `make android-avd` | Create ZFold4_dev AVD |
| `make android-emu` | Start emulator, wait for boot |
| `make android-emu-stop` | Kill emulator |
| `make android-install` | Build Rust .so + install APK |
| `make android-build-rust` | Rust only (skip Gradle) |
| `make android-log` | Tail `BrickDemo` logcat |
| `make android-devices` | List connected ADB devices |
| `make android-clean` | Delete build outputs + jniLibs |
| `make test` | Run Rust host tests |

## Architecture

```
Counter (Rust)          android.rs                  BrickSignalBridge.kt
  Signal<i32>  ──set()──► SIGNAL_QUEUE  ──drain()──► MutableState<String>
                                                           │
  register_action()  ◄──dispatch()──  BrickJni.dispatch() │
  increment() / decrement()                    Compose recomposes Text
```

1. `MainActivity.brickDemoBlueprintBytes()` → Rust builds `column![p(count), button, button]`,
   wires `count` signal to drain queue, registers actions, returns prost-encoded blueprint.
2. `BrickRenderer.BrickNode()` renders the blueprint as Material 3 Compose widgets.
3. `BrickSignalBridge.startDrainLoop()` hooks `Choreographer` — every VSYNC drains the queue,
   updates `MutableState`, Compose recomposes the `Text` showing the count.
4. Button tap → `BrickJni.dispatch("increment")` → Rust increments `count` signal →
   next frame drain picks up the new value → `Text` updates.
