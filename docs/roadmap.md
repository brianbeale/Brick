---
name: roadmap
description: "Full ambition roadmap for Brick — reactive UI framework scaling from web DOM to SSR to native platform rendering on iOS, Android, macOS, Windows"
metadata:
  node_type: memory
  type: project
  originSessionId: c02b6b4e-7c8d-4bb0-9c6f-9e8453344ce2
---

Goal: a reactive UI framework built on clean abstractions that scale from browser DOM to server-side rendering to genuinely native platform widgets — one macro triad, one reactive core, no runtime bridge, every target.

## Versioning intent
- **v0.1** — internal milestone only; not published
- **v0.2** — after Phase 11 (SSR): full-stack capable; first candidate for docs + publish
- **v1.0** — Android + web parity achieved (Phases 12–14 complete); cross-platform release with docs site and announcement
- **iOS (Phase 15)** — deferred; blocked on Mac hardware acquisition; not a v1.0 blocker

---

## Phase 1 — API Foundations ✓
- [x] `cascade()`, `#[from]`, `#[default]`
- [x] `#[view]` → `impl IntoComponent` + `mount()`
- [x] Slots — `Slot`, `slot!`, `#[slot]`

## Phase 2 — List Rendering ✓
- [x] `List<T>`, `ViewList<T>`, `list!` macro
- [x] `.remove()`, `.bind()`, `BrickAction` / `{Name}View`
- [x] TodoList demo

## Phase 3 — Global State and Context ✓
- [x] `Signal<T>`, `#[store]`, `#[global]`
- [x] `stores!`, `globals!`
- [ ] Context API — deferred; RAII stack design exists in memory [[context-design]], re-add when compound components drive the need
- [ ] Store/global demo

## Phase 3.5 — API Consistency ✓
- [x] `BrickAction { name: &'static str, event: &'static str }` — carry event type alongside action name
- [x] Generate a `BrickAction` field for ALL controller methods, not just zero-arg
- [x] `.trigger(&my.update_depart)` works for parameterized methods — reads embedded event type to register `input` vs `click` listener
- [x] `.on(str)` demoted to `.on_event(str)` escape hatch only; no longer needed in practice

## Phase 4 — Async and Data Fetching ✓
- [x] `wasm-bindgen-futures` integration — `spawn_local` re-exported
- [x] `Load<T, E>` enum — `Idle | Loading | Loaded(T) | Failed(E)` typed async state machine with `is_loading/is_loaded/is_failed/loaded/err` helpers
- [x] `Signal::map(f)` — derived/computed signals
- [x] `resolve!` macro — retired; replaced by composable `load!`/`wait!`/`catch!`/`idle!`
- [x] `load!(signal, |v| ...)` — renders Loaded arm only; nothing for other states
- [x] `wait!(signal, fallback, content)` — gates content behind loading fallback
- [x] `catch!(signal, |err| fallback, content)` — gates content behind error fallback; `err` is `Signal<String>`
- [x] `idle!(signal, content)` — shows content only in Load::Idle; pairs with `Resource::lazy()`
- [x] `Resource<T>` — typed async wrapper; `new()` auto-fetches, `lazy()` starts Idle; `refetch()`; first-class `#[model]` field type
- [x] `#[on(interval, ms)]` — generates `setInterval` in `attach_listeners`
- [x] `#[on(mount)]` — runs callback immediately at attach time; bug fixed (was emitting wrong variable name)
- [x] 7GUIs Timer demo (#2) — `tick()` via interval, `set_duration`, `reset`; 7 unit tests

## Phase 5 — Routing (v0.1 milestone) ✓
- [x] Client-side router — history API (pushState + popstate), typed route enums
- [x] Route parameters — `#[path("/:id")]` typed extraction into enum variant fields
- [x] Nested routes — `UsersRoute(ChildRoute)` + `#[routes(parent_enum=..., parent_variant=...)]`
- [x] `.link()` on routes, builders, ZST fields, `&str`; `navigate(route)`, `app_route()` signal
- [x] `subroute!` + `ViewMapped<T>` — reactive page dispatch
- [x] Route guards — `RwRoute::guard(|route| Option<Route>)` on top-level enums; chain runs inside `navigate()` before signal update

## Phase 6 — Forms ✓
- [x] `#[validate(...)]` attribute — required, min_length/max_length, email, url, alpha, alphanumeric, min/max (length-aware for String/Vec), custom validators via any `Fn(&T) -> Result<(), String>`
- [x] Companion signals — `{field}_error: Signal<String>`, `{field}_touched: Signal<bool>`, `{field}_dirty: Signal<bool>`
- [x] `validate_all()` / `validate_{field}()` — always generated; `#[on(submit)]` automatically gates on `validate_all()`
- [x] `Validate<T>` trait + `IsPresent`, `IsPositive`, `IsAlpha` helper traits in `src/validation.rs`
- [x] `.touch()`, `.dirty()`, `.submit(&action)` builder methods on view elements
- [x] `BrickForm` component — renders `<form>`, wires enter-key via programmatic click on `[data-brick-action]`
- [x] Aggregate form-state signals — `valid: Signal<bool>`, `dirty: Signal<bool>`, `touched: Signal<bool>` generated on any model with `#[validate]` fields. `valid` = touched && all errors empty. `validate_all()` forces touched=true. Validated fields default to `T::default()` in cascade so field validators can read without panicking.
- [x] Contact form demo — `src/examples/contact_form.rs`

## Phase 7 — Visual Rendering
SVG, Canvas, and the 7GUIs Circle Drawer. Makes Brick viable for data viz, diagrams,
games, and any app that needs drawing primitives beyond HTML elements.

**SVG core infrastructure** ✓
- [x] `PathData` builder — full SVG path DSL, all absolute + relative commands (`src/view_components/svg/path.rs`)
- [x] `Transform` builder — composable transforms with `then_*` chaining (`src/view_components/svg/transform.rs`)
- [x] `SvgColor` + `IntoSvgColor` trait — accepts `&str`, `String`, `SvgColor`, `Color`, `GradientRef`, `PatternRef` (`src/view_components/svg/color.rs`)
- [x] `IntoSvgTransform` trait — accepts `Transform`, `&str`, `String`
- [x] `SvgLeaf` + `SvgContainer` backing structs; `HasSvgLeaf` trait; `IntoSvgF64` trait (accepts `f64`/`f32`/`i32`/`usize`/`Signal<f64>`) (`src/view_components/svg/core.rs`)
- [x] `StrokeBuilder<T>` / `FillBuilder<T>` sub-builders + `SvgElemMethods` blanket trait (`src/view_components/svg/presentation.rs`)

**SVG element library** ✓
- [x] Containers: `svg_root`, `svg_group`, `svg_defs`, `svg_symbol`, `svg_use`, `svg_marker` (`src/view_components/svg/containers.rs`)
- [x] Shapes: `svg_circle`, `svg_rect`, `svg_ellipse`, `svg_line`, `svg_polyline`, `svg_polygon`, `svg_path` (`src/view_components/svg/shapes.rs`)
- [x] Text: `svg_text`, `svg_tspan` (`src/view_components/svg/text.rs`)
- [x] Effects: `linear_gradient`, `radial_gradient`, `stop`, `clip_path`, `svg_filter`, `svg_mask`, `svg_pattern`; filter primitives: `fe_gaussian_blur`, `fe_color_matrix`, `fe_blend`, `fe_composite`, `fe_offset`, `fe_merge`, `fe_flood`, `fe_turbulence` (`src/view_components/svg/effects.rs`)
- [x] SVG animation: `animate`, `animate_transform`, `animate_motion` with full SMIL attribute set (`src/view_components/svg/animate.rs`)
- [x] 105 SVG tests; total test count: 336

**Reactive geometry** ✓
- [x] `Signal<f64>` accepted anywhere geometry takes `impl IntoSvgF64`; registers DOM observer on reactive attrs via identity class

**Presentation sub-builder pattern** ✓ (user-requested `.stroke(x).width(y)`)
- [x] `.stroke(color)` → `StrokeBuilder<T>` with `.width()`, `.opacity()`, `.linecap()`, `.linejoin()`, `.dasharray()`, `.dashoffset()`
- [x] `.fill(color)` → `FillBuilder<T>` with `.opacity()`, `.rule()`
- [x] Both builders implement `IntoComponent` and `ViewComponent` for seamless use in children

**Canvas:** ✓
- [x] `canvas_sized(width: u32, height: u32)` convenience helper — `src/view_components/leafs/media.rs`

**Reactive SVG subtrees:** ✓
- [x] `SvgViewMapped<T>` — like `ViewMapped<T>` but uses `<g data-brick-svg-mapped="N">` container; valid inside SVG documents (`src/view_components/view_helpers.rs`)
- [x] Event delegation upgraded to use `closest("[data-brick-action]")` so bubbled SVG events resolve correctly
- [x] `SvgElemMethods::trigger()` — sets `data-brick-action` on any SVG element

**Animation frame:** ✓
- [x] `#[on(animation_frame)]` controller attribute — fires on every `requestAnimationFrame` tick via recursive Rc<RefCell<Option<Closure>>> pattern; handle cancelled in `detach()` via `cancelAnimationFrame`; enables game loops and particle systems
- [x] `ViewComposite` extended with `raf_callbacks` + `raf_handles` fields
- [x] `make_composite!` updated to 7-argument form including `raf_callbacks`
- [x] `controller_methods()` return type updated to 5-tuple (added `Vec<Rc<dyn Fn()>>` for RAF)

**7GUIs Circle Drawer** ✓
- [x] `CircleDrawer` demo — `src/examples/circle_drawer.rs`; click to create circles, click-within to select + open radius dialog, full undo/redo for add+resize actions; 12 unit tests; 352 total tests
- [x] `Circle { x, y, r, selected: bool }` — embeds selection state to avoid diamond reactivity
- [x] Undo/redo: add-circle pushes immediately; radius change pushed on dialog close only if changed; uses `Signal<Vec<Vec<Circle>>>`
- [x] DomRect + DomRectReadOnly + SvgElement added to Cargo.toml web-sys features

## Phase 8 — Motion & Animation ✓
CSS animation ergonomics from Level 2 (enter/exit) through Level 7 (View Transitions).
Makes Brick apps feel polished and modern. `style!` already handles static CSS; this
phase closes the gaps that require framework cooperation.

**Level 2 — Enter/exit animations:** ✓
- [x] `.enter("cls")` / `.exit("cls")` builder on `ViewConditional` — adds class before swap, listens for `animationend`/`transitionend`, then performs mount swap

**Level 3 — Keyframe builder:** ✓
- [x] `keyframes!` macro — generates `@keyframes` blocks from Rust structs; `proc_macs/src/keyframes.rs`; `src/view_components/keyframes.rs`

**Level 4 — Spring physics:** ✓
- [x] `Spring` — damped harmonic oscillator driven by `requestAnimationFrame`; `src/state_mgmt/spring.rs`
- [x] `Spring::follow(signal, stiffness, damping)` wraps any `Signal<f64>`; readable as `Signal<f64>`

**Level 5 — Stagger helpers:** ✓
- [x] `stagger(index: usize, step: Ms) -> Ms` — computes per-item animation delay
- [x] `list!(path, |item, i| ...)` — optional index parameter; `flip!` updated the same way

**Level 6 — FLIP animations:** ✓
- [x] `FlipList<T>` — mirrors `ViewList<T>`, adds `data-flip="true"` on items; pointer-based identity via `bli{hex}` CSS class reused from `ViewList`; `src/view_components/flip_list.rs`
- [x] On remove: snapshot rects → remove → snapshot after (forces reflow) → apply inverted `transform: translate(dx,dy)` with `transition: none` → force reflow → remove overrides (CSS transition plays)
- [x] `flip!` macro — two-form: `flip!(list, |item| ...)` and `flip!(list, 300, |item| ...)` with duration

**Level 7 — View Transitions API:** ✓
- [x] `start_view_transition(callback)` — runtime feature detection via `js_sys::Reflect`; graceful fallback to direct call; `src/view_components/mod.rs`
- [x] `ViewConditional::transition()` — implies mount mode, wraps swap in `startViewTransition`
- [x] `ViewMapped::transition()` — wraps re-render in `startViewTransition`
- [x] `.transition("name")` on element builders — sets `view-transition-name` CSS property for cross-route matched-element animations

## Phase 9 — Browser Platform APIs ✓
A holistic web framework owns the full browser platform. Added `serde` + `serde_json`
dependencies; all APIs live in `src/browser/`.

**Storage:** ✓
- [x] `BrickStorage::get::<T>(key)` / `set(key, &val)` / `remove(key)` — typed localStorage free functions; serde-backed; `src/browser/storage.rs`
- [x] `#[persist("key")]` field annotation on `#[store]` / `#[global]` — macro-generated load-on-init + save-on-change; `proc_macs/src/model.rs` + `FieldKind::Persist(String)`
- [ ] IndexedDB async wrapper — deferred (async + transactional, deserves own design pass)

**Network:** ✓
- [x] `fetch(url).header(k,v).get::<T>()` / `.post::<B,T>(&body)` / `.put` / `.patch` / `.delete` — `FetchBuilder`; `src/browser/fetch.rs`
- [x] `BrickSocket::<T>::connect(url)` — reactive WebSocket; `.signal() -> Signal<Option<T>>`, `.send(&msg)`, `.close()`; JSON-serialized; `src/browser/socket.rs`
- [x] `BrickEventSource::<T>::connect(url)` — reactive SSE; `.signal() -> Signal<Option<T>>`, `.close()`; `src/browser/event_source.rs`

**Device / OS:** ✓
- [x] `clipboard::read() -> Future<Result<String, String>>` / `clipboard::write(text)` — `src/browser/clipboard.rs`
- [x] `geolocation::get() -> Future<Result<Coords, String>>` — callback→Promise→JsFuture; full `Coords` struct; `src/browser/geolocation.rs`
- [x] `files::from_event(&event) -> Vec<BrickFile>` — handles `HtmlInputElement` + `DragEvent`; `BrickFile::read() -> Future<Result<Vec<u8>, String>>`; `src/browser/files.rs`
- [ ] Push notifications — deferred; VAPID signing is server-side, full flow crosses client/server/push-service boundary; document pattern rather than wrap it

**Compute — Web Workers:**
- [ ] `BrickWorker<In, Out>` — wraps a Web Worker, exposes `Signal<Option<Out>>`
- **Constraint**: worker must be a separate WASM binary with its own `wasm-bindgen` entry point; no shared memory without `SharedArrayBuffer` (requires COOP/COEP headers). This is a toolchain problem as much as a library problem — needs a `trunk`-style build step that produces two artifacts. Strong Rust win potential: offload CPU-heavy work (image processing, parsing, crypto) to a background thread entirely in safe Rust, zero JS glue in the worker itself.

**PWA — Service Workers:**
- [ ] Cache strategy builders — emit a JS service worker file as a build artifact
- [ ] `service_worker::register(path)` — thin runtime registration wrapper
- **Constraint**: service workers cannot run WASM (different global scope, no DOM). The worker itself must be JS. However, Brick can own the *generation* of that JS file at build time — macro or CLI tool emits a preconfigured `sw.js` from a Rust strategy struct. JS-as-output is not the same as JS-as-input. Not insurmountable.
- [ ] `manifest.json` generation from a Rust struct — simpler, pure build artifact, no runtime constraint

## Phase 10 — Renderer Abstraction (architectural linchpin) ✓
This phase earns everything that follows. The goal is to make the rendering target a
parameter, not an assumption. Every subsequent phase — SSR, native — depends on this
being right.

**Design decisions (approved):**
- cfg-based renderer selection (NOT cargo features): `--cfg brick_dom` sets DomRenderer, no flag → HtmlRenderer (default for `cargo test`)
- ZST BrickRenderer implementors: zero runtime overhead, pure static dispatch
- Single-phase render: `render() -> R::Node` creates nodes AND wires observers; no separate attach_listeners
- DomNode(web_sys::Node) newtype for DOM target; HtmlNode(Rc<RefCell<HtmlNodeInner>>) tree for tests/SSR
- `BrickComponent<R>` trait exists alongside legacy `ViewComponent` during transition; detach() absent until ViewComponent fully retired
- Bridge approach: complex types with Box<dyn ViewComponent> children use HtmlNode::raw(self.html()) until children migrated

**Infrastructure: ✓**
- [x] `src/renderer/mod.rs` — `BrickRenderer` trait, `BrickEvent` enum (Click, InputString, InputBool, InputNumber), cfg-based `Renderer` type alias; custom cfg keys declared in Cargo.toml
- [x] `src/renderer/html.rs` — `HtmlNode` (Rc<RefCell<HtmlNodeInner>> tree with Element/Text/RawHtml variants), `HtmlRenderer` with full BrickRenderer impl, `to_html_string()`; 15 unit tests
- [x] `src/renderer/dom.rs` — `DomNode`(web_sys::Node) newtype, `DomRenderer` with full web_sys implementation; on_event dispatches BrickEvent variants; cfg(brick_dom) only

**BrickComponent trait and impls: ✓**
- [x] `BrickComponent` trait (non-generic, renderer fixed at compile time via `Renderer` alias)
- [x] `IntoBrickComponent` type-erasure trait with blanket impls
- [x] All container types migrated to `Vec<Box<dyn BrickComponent>>` — ViewComposite, BrickContainer, BrickRow, ReactiveDiv, ViewList, FlipList, ViewResolve, ViewMapped, SvgViewMapped, SvgLeaf, SvgContainer
- [x] `IntoComponent::into_component()` returns `Box<dyn BrickComponent>` everywhere — all leaf elements, SVG types, slot, view proc macro, routes proc macro, example files
- [x] `ViewConditional::new()` accepts `Box<dyn BrickComponent>` via `BrickToCond` bridge (internal Rc<dyn ViewComponent> storage preserved)
- [x] `nothing()` returns `Box<dyn BrickComponent>`
- [x] `ViewLeafText::render()` uses `HtmlNode::raw()` for text_content (preserves live! HTML spans)
- [x] 27+ parity tests verify render().to_html_string() == html() across all types
- [x] 515 tests pass (0 failures)

**Remaining Phase 10 work:**
- [x] Retire legacy `ViewComponent` trait — remove `html()` + `attach_listeners()` + `detach()`, replace call sites with `render().to_html_string()`
- [x] Remove `Render` trait; `#[view]` generates `impl IntoComponent` directly; `mount()` moved to `IntoComponent` default method; `make_composite!` returns `Box<dyn Brick>`
- [x] Inline event wiring during `render_into` via `Renderer::on_event()` — replaces CSS-class-query document-level delegation; thread-local `RENDER_SCOPE_LISTENERS` propagates action listeners through composite children; `attach_listeners` is now a no-op on all types; `BrickEvent` updated with `ClickAt(f64, f64)`; `FromBrickEvent` replaces `FromInputEvent`; `#[on(click, coords)]` extractor added for mouse-position handlers
- [x] CRUD refactored from raw DOM event delegation to `#[on(watch, selected_ptr)]` + `.set(&selected_ptr, ptr)` per list item — no DOM access required
- [x] Circle drawer refactored from `#[on(click, raw)]` + `web_sys::Event` to `#[on(click, coords)]` + `query_selector("svg.circle-canvas")`
- [x] Fragment support in HtmlNode — `HtmlNodeInner::Fragment(Vec<HtmlNode>)` serializes as concatenated children; `Renderer::fragment()` and `is_fragment()` implemented
- [x] Eliminate all `#[cfg(not(test))]` guards once DomRenderer handles the DOM-specific path
- [x] `.cargo/config.toml` web target sets `--cfg brick_dom` via RUSTFLAGS

## Phase 11 — SSR and Full-Stack (v0.2 milestone)
Depends on `HtmlRenderer` from Phase 10.

- [x] Server-side render to HTML string — synchronous, runs outside WASM, no browser required
- [x] Hydration — reattach `DomRenderer` listeners to SSR HTML without re-rendering; stable IDs make this exact
- [x] `#[server_fn]` macro — shared type signature; fetch call on client (`fetch_bytes` POST), real handler on server (`inventory::submit!` registration); prost and JSON encoding modes; `#[server_fn]` alias added alongside `#[server]`
- [x] `brick::Message` derive — hash-based stable prost tags (`fnv32(name) & 0x1FFF_FFFF`), type inference from Rust types (no manual `#[prost(type, tag)]`), generates `Default` + `Debug` + `impl prost::Message` via internal proxy struct; supports scalars, `Vec<u8>`, `Vec<T>`, `Option<T>`; 11 unit tests + end-to-end integration test
- [ ] Streaming SSR — flush shell immediately, stream content fragments as Resources resolve

## Phase 12 — Native Foundation ✓
Proved the JNI + Jetpack Compose approach and established the portable layer architecture.

**Research spike — Android via JNI + Jetpack Compose:** ✓
- [x] Portable layer: `BlueprintNode` enum, `PortableView` trait, `encode_blueprint` → prost-encoded `BrickViewBlueprint`
- [x] JNI bridge: `libbrick.so` via `cargo-ndk`; `BrickJni.kt` (`load`, `dispatch`, `drainQueue`, `setupSignalBuffer`, `clearAndroidState`)
- [x] `BrickSignalBridge.kt` — shared `ByteBuffer` drain loop via Choreographer; typed wire format (`SignalEncoding` trait, 14 impls); `@Volatile isActive` lifecycle
- [x] `BrickRenderer.kt` — recursive `BrickNode` composable; all 7 node types (Text, Button, Input, Column, Row, Scroll, Toggle) mapping to Material 3
- [x] Decision: JNI approach succeeds cleanly. `BrickRenderer` generalizes via blueprint layer rather than direct trait impl. Proceed to Phase 13.

**Cross-platform widget vocabulary:** ✓
- [x] Portable primitives: `p()`, `button()`, `input()`, `toggle()`, `column()`, `row()`, `scroll()`, `intent_button()`, `permission_request()`
- [x] `ButtonVariant` (Primary/Secondary/Outlined/Ghost/Danger/DangerOutlined) — maps to M3 composables on Android, CSS classes on web
- [x] `TextVariant` (Body/Title/Caption/Label) — maps to M3 typography scale
- [x] `Navigator<S>` — portable back-stack navigator with `push`/`pop`/`current`; triggers full re-render via `push_rerender_signal`
- [x] `IntentType` enum — ShareText/OpenUrl/LaunchCamera/PickImage/OpenSettings/ComposeEmail → Android Intents
- [x] `#[cross_platform]` annotation and `platform_match!` macro — superseded by the portable layer; not needed

**Semantic builder → platform theme:** ✓
- [x] `.primary()`, `.danger()`, `.ghost()` etc. carry semantic intent — Android maps to M3 `Button`/`FilledTonalButton`/`TextButton`; web maps to CSS classes
- [x] `BrickDarkColors` — full M3 `darkColorScheme` mapped from Brick brand palette tokens
- [x] Typography scale, spacing via M3 defaults on Android

**Layout engine strategy:** ✓ (resolved without Yoga)
- [x] Compose's own `Column`/`Row`/`Arrangement.spacedBy` is sufficient — no Yoga needed

**Build toolchain:** ✓
- [x] `cargo-ndk` → `aarch64-linux-android` → `android/demo/src/main/jniLibs`
- [x] Gradle integration: `make android-install` = `cargo ndk build --release` + `./gradlew :demo:installDebug`
- [x] iOS toolchain researched (cargo-mobile2 v0.22.1, objc2 0.6, aarch64-apple-ios-sim) — deferred to Phase 14 pending Mac hardware

## Phase 13 — Android Demo Suite + Signal Quality ✓
Expanded from single counter to 11 demos; hardened signal encoding and drain loop.

- [x] `SignalEncoding` trait — 14 typed impls; shared `ByteBuffer` zero-copy drain replaces string serialization
- [x] `dispatchInt` / `dispatchBool` — typed JNI dispatch; `INT_ACTION_TABLE` / `BOOL_ACTION_TABLE` thread_locals
- [x] `trigger_int(action, i32)` on `PortableButton` — per-item index dispatch for lists
- [x] `toggle()` portable primitive with `bind(&Signal<bool>)` + `on_change`
- [x] `Navigator<S>` portable routing — `push_rerender_signal` drives full blueprint re-fetch
- [x] Drain loop leak fixed — `@Volatile isActive` + `DisposableEffect` stop old bridges on tab switch
- [x] Action re-registration — all 11 JNI entry points call `register_android_actions` outside model-init guard so actions survive `clearAndroidState`
- [x] 11 Android demos: Counter, Timer, Contact Form, TodoList, Thermometer, Flight Booker, Double Counter, TodoMVC, CRUD, Nav Demo, Theme Demo
- [x] `trybuild`, `cargo deny`, `cargo doc` CI checks added
- [x] 584 passing tests

## Phase 14 — Android Polish + Style System ✓
Visual quality, theming, and a native style testing framework.

- [x] `BrickDarkColors` — full M3 dark scheme mapped from brand palette; `primary=#D64236`, `error/danger=#BB0033`
- [x] `ButtonVariant::Danger` (crimson filled) + `DangerOutlined` (crimson outlined border)
- [x] `TextVariant` label color fixed — explicit `color = MaterialTheme.colorScheme.onSurface` on all Text composables
- [x] `StyleCatalog.kt` — native Kotlin style testing composable: color swatches for all M3 roles, all 15 typography styles, all 6 button variants + disabled states, input/toggle controls; wired as "Styles" tab
- [x] Enhanced `android_theme_demo.rs` — interactive portable component playground: live input with reactive display, toggle with label flip, all 6 button variants, full typography ladder
- [x] Pixel art brick icon (`tools/icon_gen/`) — pure-Rust PNG encoder (CRC32, Adler32, deflate stored); 3D single brick design; generates all 5 Android mipmap densities
- [x] Edge-to-edge display — `enableEdgeToEdge()` + `statusBarsPadding()`
- [x] App icon wired via `android:icon="@mipmap/ic_launcher"`

## Phase 15 — iOS Native Renderer *(blocked on Mac hardware)*
- [ ] `IosRenderer` implementing `BrickRenderer` via `objc2` UIKit bindings
- [ ] Full portable component library rendered as UIKit widgets (UILabel, UIButton, UITextField, UIStackView, UIScrollView, UISwitch)
- [ ] Touch events and native iOS gestures wired to controller methods via `UIAction` blocks (iOS 14+)
- [ ] `UINavigationController` integration with `Navigator<S>` (push/pop maps to route changes)
- [ ] VoiceOver accessibility — UIAccessibility labels derived from component semantics
- [ ] Safe area, notch, dynamic type — platform conventions respected automatically
- [ ] App Store distribution workflow documented
- [ ] Research complete: cargo-mobile2 v0.22.1, objc2 0.6, aarch64-apple-ios-sim deploy pattern; see phase12_ios_build_pipeline and phase12_objc2_uikit memory files

## Phase 16 — Desktop Native Renderers
- [ ] `MacosRenderer` — AppKit via `objc2`; menu bar, system fonts, native scroll, dark mode
- [ ] `WindowsRenderer` — Win32 / WinUI via `windows-rs` crate
- [ ] `LinuxRenderer` — GTK4 via `gtk4-rs`
- [ ] Tauri Desktop path — official guide for users who want OS WebView instead of native widgets (already works today)

---

## Quality & Observability (parallel track)
Independent of phase work — these can be picked up any time.

**Observability:**
- [ ] `tracing` crate integration — structured logging in `#[server]` functions server-side
- [ ] `tracing-wasm` — route `tracing` events to browser console via `console.log`; drop-in subscriber for WASM target
- [ ] `tracing-subscriber` — server-side subscriber with JSON or pretty formatter for `#[server]` fn request logging

**Proc macro testing:**
- [ ] `trybuild` — compile-test `#[server]` and `#[model]` error paths; verify that 2+ prost args, invalid encoding names, invalid validate attrs, etc. emit correct compiler errors at the right span

**Property-based testing:**
- [ ] `proptest` — fuzz the reactive core: signal propagation chains, computed diamond graphs, `Load` state machines; catch edge cases that unit tests miss

**Dependency hygiene:**
- [ ] `cargo deny` — audit deps for known CVEs, duplicate versions, license compliance; one config file + CI step

**Documentation:**
- [ ] `cargo doc --no-deps` check in CI — catches broken doc links before they accumulate
- [ ] `#![deny(missing_docs)]` on public API items — applied incrementally starting with `src/lib.rs` re-exports

---

## Ongoing / Parallel
- [ ] Cross-platform component library — `TextInput`, `NumberInput`, `Checkbox`, `Select`, `Textarea`, `Modal`, `Toast` (web implementations first, native mappings added in Phase 13/14)
- [ ] Accessibility — ARIA on web; platform accessibility in native phases
- [ ] External CSS file injection — `style!(include "component.css")` reads a `.css` file at compile time and injects it as scoped component CSS. Enables editor tooling, syntax highlighting, linting. Consider: CSS Modules-style class hashing, PostCSS-like transforms at macro expansion time, auto-discovery of co-located `.css` files.
- [ ] Head / metadata management — reactive `<title>`, meta tags, canonical URLs (web)
- [ ] i18n foundation — locale-aware `live!` format, pluralization, RTL layout
- [ ] Virtual list windowing — once perf is observable at scale
- [ ] DevTools — browser panel showing component tree + reactive graph; native inspector TBD
- [ ] Docs site + crates.io publish — deferred to v1.0 or beyond; needs dedicated time and a different kind of preparation that doesn't synergize with active framework development

---

## Completed
- [x] Reactive core — `state!`, `set!`, `Subject`/`Observer`, `Effect`, computed
- [x] `#[model]`, `#[controller]`, `#[view]` macro triad
- [x] `live!`, `when!`, `list!`, `row!`, `slot!`
- [x] `div {}` DSL in `#[view]` — `div { class("foo"), children }` / `div { class(signal), children }` replaces `div_c!` and `reactive_div!`
- [x] Full HTML element library + view builders (variants, sizes, spacing, typography)
- [x] `css!` / `style!` — component-scoped CSS injection with `@layer` architecture
- [x] Theme system — CSS variables, dark theme, brick-red palette
- [x] 231 unit tests + 4 browser integration tests + GitHub Actions CI
- [x] TodoMVC, Flight Booker (7GUIs #3), CRUD (7GUIs #5), Timer (7GUIs #2), Contact Form demos
- [x] 7GUIs at 5/7 — Counter, Temp Converter, Flight Booker, Timer, CRUD
- [x] `ReactiveDiv`, `CheckedObserver`, `ClassObserver`
- [x] Fixed ViewList double render_item (stable observer IDs)

## Deferred / Future assessment
- [ ] 7GUIs Circle Drawer — now planned in Phase 7 (SVG-based, not canvas)
- [ ] 7GUIs Cells — spreadsheet; likely out of scope
- [ ] RealWorld SPA demo — Phase 9 is complete; next major milestone before Phase 10
- [ ] BlitzRenderer — Stylo + Vello native HTML/CSS rendering; fallback if native widget FFI spike (Phase 12) reveals blockers; revisit after v1.0
- [ ] WASM Component Model — future standard for running Brick components in any WASM host
