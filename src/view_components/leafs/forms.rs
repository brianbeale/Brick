#![allow(dead_code)]
use super::ViewLeafText;

// Form container and non-interactive elements
tag_funcs!(
    form, textarea, select, option, label, fieldset, legend, datalist, optgroup, output
);
// Void input — no text content, no closing tag
tag_funcs_void!(input);

pub fn button<T: std::fmt::Display + ?Sized>(text: &T) -> Box<ViewLeafText> {
    Box::new(ViewLeafText {
        tag: "button",
        class_name: String::new(),
        text_content: text.to_string(),
        action_name: None,
        attrs: Vec::new(),
        is_void: false,
        bind_class: None,
        bind_initial: None,
        bind_handler: None,
        click_class: None,
        click_handler: None,
        toggle_class: None,
        toggle_handler: None,
        brick_classes: Vec::new(),
        inline_style: None,
        style_signal_class: None,
        style_signal_initial: None,
        live_classes: Vec::new(),
        touch_class: None,
        touch_handler: None,
        dirty_class: None,
        dirty_handler: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn button_renders_text_no_class() {
        assert_eq!(rh(&*button("+")), "<button>+</button>");
    }

    #[test]
    fn button_c_sets_class() {
        assert_eq!(
            rh(&*button("+").c("increment")),
            r#"<button class="increment">+</button>"#
        );
    }

    #[test]
    fn button_c_appends_multiple_classes() {
        assert_eq!(
            rh(&*button("Go").c("tab-btn").c("active")),
            r#"<button class="tab-btn active">Go</button>"#
        );
    }

    #[test]
    fn button_on_wires_action() {
        assert_eq!(
            rh(&*button("+").on_event("increment")),
            r#"<button data-brick-action="increment">+</button>"#
        );
    }

    #[test]
    fn button_c_and_on_are_independent() {
        assert_eq!(
            rh(&*button("+").c("btn").on_event("increment")),
            r#"<button class="btn" data-brick-action="increment">+</button>"#
        );
    }

    #[test]
    fn input_is_void() {
        assert_eq!(rh(&*input()), "<input>");
    }

    #[test]
    fn input_with_type_and_placeholder() {
        assert_eq!(
            rh(&*input().attr("type", "text").attr("placeholder", "Name")),
            r#"<input type="text" placeholder="Name">"#
        );
    }

    #[test]
    fn textarea_renders() {
        assert_eq!(
            rh(&*textarea("").c("notes")),
            r#"<textarea class="notes"></textarea>"#
        );
    }

    #[test]
    fn select_with_option() {
        assert_eq!(
            rh(&*option("Choice A").attr("value", "a")),
            r#"<option value="a">Choice A</option>"#
        );
    }
}
