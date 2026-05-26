use crate::view_components::*;

#[model]
pub struct Counter {
    pub count: isize,
}

#[view(Counter)]
fn render() -> Box<ViewComposite> {
    style! {
        .brick-row { display: inline-flex; gap: 0.5rem; }
        .brick-row button { padding: 0.3rem 0.4rem; }
    }
    children! {
        row! {
            button("−").click(&my.count, |v| v - 1).secondary(),
            p(live!("{my.count}")).text_xl(),
            button("+").click(&my.count, |v| v + 1).primary(),
        },
    }
}
