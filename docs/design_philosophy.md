---
name: design-philosophy
description: "Brick's core design decisions, API conventions, and architectural intent"
metadata: 
  node_type: memory
  type: project
  originSessionId: c02b6b4e-7c8d-4bb0-9c6f-9e8453344ce2
---

Brick's defining characteristic is enforced MVC structure per component via three proc macros. Each component is exactly:

```rust
#[model]   struct Counter { count: usize }
#[controller] impl Counter { fn increment(&mut self) { set!(self.count => + 1); } }
#[view(Counter)] fn render() -> Box<ViewComposite> { children! { ... } }
```

This structure is not a convention — it's structurally enforced by the macros.

**Reactivity model:** Signal-based, no virtual DOM. State updates → observers notified → Effect/SpanObserver mutates specific DOM nodes directly. No diffing.

**Key macro details:**
- `#[model]` — transforms plain struct fields into `Rc<RefCell<Box<dyn Subject<T>>>>`. Fields with `#[from(source, fn)]` are derived; `#[model]` currently generates `new()` as an interim constructor. Will be replaced by `cascade()` mechanism. Fields with `#[default(val)]` (planned) have static defaults filled by `cascade()`.
- `#[controller]` — wraps methods as event-listener closures in a HashMap; `#[on(eventtype)]` per method (default: click); typed extractors via `#[on(input, f64)]`
- `#[view]` — generates `impl Render`, makes `model: Rc<RefCell<T>>` available, generates unique instance class via `INSTANCE_COUNTER`. Will generate `impl From<T> for Box<ViewComposite>` and builder methods (`.c()` etc.) directly on the component struct.
- `live!` — reactive template macro, `"{model.field:.2}"` applies format spec to typed value, not HTML. Replaces old `react_text!`.
- `state!(val)` — wraps a value in `Rc<RefCell<Box<dyn Subject<T>>>>` via `make_state()`
- `bind!` — creates `BoundState` for bidirectional parent↔child state sharing
- `set!(self.field => val)` — updates reactive state, notifies all observers

**`cascade()` API (planned — see [[project-status]]):**
Two-phase construction for models with `#[from]` or `#[default]` fields:
```rust
// Declaration (in component file):
#[model]
pub struct Thermometer {
    celsius: f64,
    #[from(celsius, |c| c * 9.0 / 5.0 + 32.0)]
    fahrenheit: f64,
}

// Instantiation (anywhere):
Thermometer { celsius: state!(0.0), ..cascade() }
```
`cascade()` is a generic free function in the prelude; type is inferred from struct context. Primary fields in `cascade()` use `Placeholder<T>` (discarded immediately by struct update syntax). Derived fields use `LazyComputed<S,T>`. Wiring happens in `controller_methods()` before the model is wrapped in `Rc<RefCell<>>`.

**HTML elements vs application components:**
- HTML elements use function-call + builder chain: `input().attr("type","number").on("update")` — appropriate because HTML attributes are open-ended and unbounded.
- Application components use struct literal syntax: `Counter { count: state!(0) }` — appropriate because component fields are fixed and named.
- This asymmetry is intentional and reflects a real semantic distinction.
- Common HTML patterns wrapped as named components (`NumberInput`, `TextInput`, etc.) are the right layer for ergonomics — not changing the raw element API.

**Instantiation conventions:**
- Simple models (no `#[from]`/`#[default]`): struct literal — `Counter { count: state!(0) }`
- Derived-field models: struct literal + `..cascade()` — `Thermometer { celsius: state!(0.0), ..cascade() }`
- Nested components in views: struct literal, no `.render()` needed (framework handles via `From<T>`)
- Shared parent state: `bind!(model.field)` instead of `state!()`

**Aware builder wrapper pattern (standard for cross-cutting element concerns):**
When a concern (animation, accessibility, data attributes, etc.) needs to be attachable to *any* `ViewComponent` without knowing its concrete type, use a generic wrapper struct:
```rust
pub struct AnimationBuilder<T: ViewComponent> {
    element: T,
    // animation params...
}
impl<T: ViewComponent> ViewComponent for AnimationBuilder<T> { ... }
impl<T: ViewComponent + 'static> IntoComponent for Box<AnimationBuilder<T>> { ... }
```
- The method that starts the chain lives on the concrete element type (e.g. `div.animate(&bounce)`)
- Builder methods return `Self`, so the chain stays on the wrapper type
- No terminal method needed — `ViewComponent` is implemented directly on the wrapper
- Dissolves at render time into inline style / attributes on the inner element
- Already used: `StrokeBuilder<T>`, `FillBuilder<T>` in the SVG layer; `AnimationBuilder<T>` for animation
- Never name the free constructor function the same as the element method that invokes it (e.g. free fn `animation()`, element method `.animate()` — not `animate(animate(...))`)

**Design constraints (non-negotiable):**
- No constructor boilerplate in example/component files — framework generates what's needed
- No `#[allow(dead_code)]` in example files — examples must look like real user code
- Struct literal form preferred; `::new()` only as interim bridge until `cascade()` exists
- `cascade()` covers both `#[from]` derived fields and `#[default(val)]` config fields — one call site for all framework-managed fields (Rust only allows one `..expr` per struct literal)
- No Co-Authored-By in git commits
