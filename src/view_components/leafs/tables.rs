#![allow(dead_code)]
// Table elements
tag_funcs!(table, thead, tbody, tfoot, tr, th, td, caption, colgroup);
tag_funcs_void!(col);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn th_renders() {
        assert_eq!(rh(&*th("Header")), "<th>Header</th>");
    }

    #[test]
    fn td_renders() {
        assert_eq!(rh(&*td("Cell")), "<td>Cell</td>");
    }

    #[test]
    fn th_with_colspan() {
        assert_eq!(
            rh(&*th("Span").attr("colspan", "2")),
            r#"<th colspan="2">Span</th>"#
        );
    }

    #[test]
    fn col_is_void() {
        assert_eq!(rh(&*col()), "<col>");
    }

    #[test]
    fn col_with_span() {
        assert_eq!(rh(&*col().attr("span", "3")), r#"<col span="3">"#);
    }
}
