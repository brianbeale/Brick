use brick::{button, h1, p, ssr};

fn main() {
    let html = ssr::render(p("hello from SSR"));
    assert!(html == "<p>hello from SSR</p>", "got: {}", html);

    let html = ssr::render(h1("Title"));
    assert!(html == "<h1>Title</h1>", "got: {}", html);

    let html = ssr::render(button("Click me"));
    assert!(html.contains("Click me"), "got: {}", html);

    println!("SSR smoke test passed.");
    println!("  p:      {}", ssr::render(p("hello from SSR")));
    println!("  h1:     {}", ssr::render(h1("Title")));
    println!("  button: {}", ssr::render(button("Click me")));
}
