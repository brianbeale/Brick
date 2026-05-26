use crate::view_components::*;

#[model]
pub struct Variants {
    pub clicks: isize,
}

#[view(Variants)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("Button Variants"),
        row! {
            button("Primary").primary(),
            button("Secondary").secondary(),
            button("Ghost").ghost(),
            button("Danger").danger(),
            button("Success").success(),
        },
        h2("Sizes"),
        row! {
            button("SM").secondary().sm(),
            button("Normal").secondary(),
            button("LG").secondary().lg(),
        },
        button("Full Width").primary().full_width(),
        row! {
            button("Ghost — track clicks").click(&my.clicks, |v| v + 1).ghost(),
            p(live!("Clicked {my.clicks}×")).muted(),
        },
    }
}
