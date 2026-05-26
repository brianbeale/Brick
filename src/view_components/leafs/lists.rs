#![allow(dead_code)]
// List elements
tag_funcs!(ul, ol, li, dl, dt, dd);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn li_renders() {
        assert_eq!(rh(&*li("item")), "<li>item</li>");
    }

    #[test]
    fn ul_renders() {
        assert_eq!(rh(&*ul("")), "<ul></ul>");
    }

    #[test]
    fn dl_dt_dd_render() {
        assert_eq!(rh(&*dt("term")), "<dt>term</dt>");
        assert_eq!(rh(&*dd("definition")), "<dd>definition</dd>");
    }
}
