use crate::view_components::*;

#[model]
pub struct Typography {
    pub ticks: isize,
}

#[view(Typography)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("Type Scale"),
        p("xs · 0.75rem").text_xs(),
        p("md · 1rem").text_md(),
        p("xl · 1.25rem").text_xl(),
        p("2xl · 1.5rem").text_2xl(),
        h2("Builders"),
        p("Bold weight").bold(),
        p("Monospace family").mono(),
        p("Truncated — intentionally long text will be clipped by overflow hidden")
            .truncate()
            .style("max-width: 200px"),
        button("p(Md)").click(&my.ticks, |v| v + 1).secondary().p(Space::Md),
        button("px(Lg)").click(&my.ticks, |v| v + 1).ghost().px(Space::Lg),
        p(live!("Clicks: {my.ticks}")).muted(),
    }
}
