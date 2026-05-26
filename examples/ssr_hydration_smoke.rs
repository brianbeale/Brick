use brick::{INSTANCE_COUNTER, button, h1, p, ssr};
/// SSR + hydration counter contract smoke test.
///
/// Demonstrates the three-step SSR hydration protocol:
///   1. Server renders with `ssr::render_full`, capturing `counter_start`.
///   2. Server embeds `script_tag()` in the page (`window.__brick_counter__ = N`).
///   3. Client reads N, restores `INSTANCE_COUNTER` to N, then re-renders the
///      same component tree — producing class names that match the SSR HTML.
///
/// The `#[view]` macro advances `INSTANCE_COUNTER` inside `Render::render()`.
/// The counter manipulation here simulates that so the example compiles without
/// needing `Signal` or `Render` in the public API.
///
/// Run with: cargo run --example ssr_hydration_smoke
use std::sync::atomic::Ordering;

fn main() {
    println!("=== Part 1: basic SSR ===");

    let html = ssr::render(p("hello from SSR"));
    assert_eq!(html, "<p>hello from SSR</p>");
    println!("p       : {html}");

    let html = ssr::render(h1("Page Title"));
    assert_eq!(html, "<h1>Page Title</h1>");
    println!("h1      : {html}");

    let html = ssr::render(button("Click me").c("primary"));
    assert!(html.contains("Click me") && html.contains("primary"));
    println!("button  : {html}");

    println!();
    println!("=== Part 2: counter capture (simulates #[view] component) ===");

    // Simulate what `#[view]` does: advance INSTANCE_COUNTER once per component.
    let out = ssr::render_full(|| {
        let n = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        // class_name would be e.g. "MyComponent_42"
        let class_name = format!("MyComponent_{n}");
        p(&class_name).c(class_name.as_str())
    });

    println!("counter_start : {}", out.counter_start);
    println!("counter_end   : {}", out.counter_end);
    println!("html          : {}", out.html);
    println!("script_tag    : {}", out.script_tag());
    println!();

    assert!(
        out.counter_end > out.counter_start,
        "component render must advance counter"
    );
    assert!(
        out.script_tag().contains(&out.counter_start.to_string()),
        "script_tag must embed counter_start"
    );

    println!("=== Part 3: hydration counter restore ===");

    // Client restores counter to counter_start (read from window.__brick_counter__).
    let counter_start = out.counter_start;
    INSTANCE_COUNTER.store(counter_start, Ordering::SeqCst);

    // Re-render: fetch_add returns counter_start, producing the same class name.
    let n = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let class_name = format!("MyComponent_{n}");

    println!("restored counter : {counter_start}");
    println!("client class     : {class_name}");
    println!("ssr html         : {}", out.html);

    assert!(
        out.html.contains(&class_name),
        "client class '{class_name}' not found in SSR HTML after counter restore to {counter_start}: {}",
        out.html,
    );

    println!();
    println!("SSR hydration smoke test passed.");
    println!("(Full #[view] component roundtrip tests are in src/ssr.rs)");
}
