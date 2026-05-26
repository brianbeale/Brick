#![allow(dead_code)]
// Block-level and sectioning elements
tag_funcs!(
    p, h1, h2, h3, h4, h5, h6, div, section, header, footer, nav, main, article, aside, blockquote,
    pre, figure, figcaption, details, summary
);
tag_funcs_void!(hr);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn p_renders() {
        assert_eq!(rh(&*p("hello")), "<p>hello</p>");
    }

    #[test]
    fn h1_renders() {
        assert_eq!(rh(&*h1("Title")), "<h1>Title</h1>");
    }

    #[test]
    fn h2_renders() {
        assert_eq!(rh(&*h2("Sub")), "<h2>Sub</h2>");
    }

    #[test]
    fn section_renders() {
        assert_eq!(rh(&*section("")), "<section></section>");
    }

    #[test]
    fn hr_is_void() {
        assert_eq!(rh(&*hr()), "<hr>");
    }

    #[test]
    fn hr_with_class() {
        assert_eq!(rh(&*hr().c("divider")), r#"<hr class="divider">"#);
    }
}
