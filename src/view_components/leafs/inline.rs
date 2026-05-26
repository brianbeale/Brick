#![allow(dead_code)]
// Inline text elements
tag_funcs!(
    span, a, strong, em, small, code, kbd, mark, del, ins, q, abbr, cite, sub, sup, time, var
);
tag_funcs_void!(br, wbr);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn span_renders() {
        assert_eq!(rh(&*span("text")), "<span>text</span>");
    }

    #[test]
    fn a_with_href() {
        assert_eq!(rh(&*a("Home").attr("href", "/")), r#"<a href="/">Home</a>"#);
    }

    #[test]
    fn strong_renders() {
        assert_eq!(rh(&*strong("bold")), "<strong>bold</strong>");
    }

    #[test]
    fn code_renders() {
        assert_eq!(rh(&*code("let x = 1;")), "<code>let x = 1;</code>");
    }

    #[test]
    fn br_is_void() {
        assert_eq!(rh(&*br()), "<br>");
    }

    #[test]
    fn wbr_is_void() {
        assert_eq!(rh(&*wbr()), "<wbr>");
    }
}
