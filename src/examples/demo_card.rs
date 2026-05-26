use crate::view_components::*;

#[model]
pub struct DemoCard {
    #[prop]
    pub label: &'static str,
    #[prop]
    pub source: &'static str,
    #[slot]
    pub preview: Slot,
    #[default(true)]
    pub show_preview: bool,
}

#[controller]
impl DemoCard {
    pub fn to_preview(&mut self) {
        set!(self.show_preview => true);
    }
    pub fn to_source(&mut self) {
        set!(self.show_preview => false);
    }
}

#[view(DemoCard)]
fn render() -> Box<ViewComposite> {
    style! {
        .preview-panel { padding: 1.25rem; }
    }
    children! {
        div {
            class("card-head"),
            span(my.label).c("card-label"),
            when!(my.show_preview,
                div {
                    class("tab-bar"),
                    button("Preview").c("tab-btn").c("active"),
                    button("Source").c("tab-btn").trigger(&my.to_source),
                },
                div {
                    class("tab-bar"),
                    button("Preview").c("tab-btn").trigger(&my.to_preview),
                    button("Source").c("tab-btn").c("active"),
                },
            ),
        },
        when!(my.show_preview,
            div {
                class("preview-panel"),
                my.preview,
            },
            div {
                class("source-panel"),
                pre(&format!(
                    r#"<code class="language-rust">{}</code>"#,
                    crate::view_components::escape_html(my.source)
                )),
            },
        ),
    }
}
