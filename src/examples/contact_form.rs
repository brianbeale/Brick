use crate::view_components::*;

#[model]
pub struct ContactForm {
    #[validate(required, min_length(2, "Name must be at least 2 characters"))]
    pub name: String,
    #[validate(required, email)]
    pub email: String,
    #[validate(required, min_length(10, "Message must be at least 10 characters"))]
    pub message: String,
    #[default(false)]
    pub submitted: bool,
}

#[controller]
impl ContactForm {
    #[on(submit)]
    pub fn send(&mut self) {
        self.submitted.set(true);
    }
}

#[view(ContactForm)]
fn render() -> Box<ViewComposite> {
    style! {
        .contact-form { display: flex; flex-direction: column; gap: 1rem; max-width: 480px; }
        .field { display: flex; flex-direction: column; gap: 0.25rem; }
        .field label { font-size: 0.85rem; color: var(--brick-muted); }
        .field input, .field textarea { margin-bottom: 0; }
        .field textarea { min-height: 6rem; resize: vertical; }
        .error { font-size: 0.8rem; color: var(--brick-danger, #e53e3e); min-height: 1.2rem; }
        .status-row { display: flex; align-items: center; gap: 0.75rem; }
        .dirty-badge { font-size: 0.75rem; color: var(--brick-muted); font-style: italic; }
        .success { padding: 1rem 1.25rem; background: color-mix(in srgb, var(--brick-accent) 15%, transparent); border-left: 3px solid var(--brick-accent); border-radius: 0.3rem; }
    }
    children! {
        h2("Contact Us"),
        when!(my.submitted,
            div {
                class("success"),
                p("Thanks! We'll be in touch."),
            },
            div {
                class("contact-form"),
                div {
                    class("field"),
                    label("Name"),
                    input()
                        .attr("type", "text")
                        .attr("placeholder", "Your name")
                        .bind(&my.name)
                        .touch(&my.name_touched)
                        .dirty(&my.name_dirty),
                    when!(my.name_touched,
                        p(live!("{my.name_error}")).c("error"),
                    ),
                },
                div {
                    class("field"),
                    label("Email"),
                    input()
                        .attr("type", "email")
                        .attr("placeholder", "you@example.com")
                        .bind(&my.email)
                        .touch(&my.email_touched)
                        .dirty(&my.email_dirty),
                    when!(my.email_touched,
                        p(live!("{my.email_error}")).c("error"),
                    ),
                },
                div {
                    class("field"),
                    label("Message"),
                    input()
                        .attr("type", "text")
                        .attr("placeholder", "How can we help?")
                        .bind(&my.message)
                        .touch(&my.message_touched)
                        .dirty(&my.message_dirty),
                    when!(my.message_touched,
                        p(live!("{my.message_error}")).c("error"),
                    ),
                },
                div {
                    class("status-row"),
                    when!(my.valid,
                        button("Send").primary().submit(&my.send),
                        button("Send").primary().attr("disabled", "true"),
                    ),
                    when!(my.dirty,
                        p("Unsaved changes").c("dirty-badge"),
                    ),
                },
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn form() -> ContactForm {
        ContactForm { ..cascade() }
    }

    #[test]
    fn renders_title() {
        assert!(crate::view_components::render_to_html(&*form().into_component()).contains("Contact Us"));
    }

    #[test]
    fn aggregate_signals_start_false() {
        let f = form();
        assert!(!f.valid.read());
        assert!(!f.dirty.read());
        assert!(!f.touched.read());
    }

    #[test]
    fn validate_all_sets_touched() {
        let mut f = form();
        f.validate_all();
        assert!(f.touched.read());
    }

    #[test]
    fn validate_all_fails_on_empty_form() {
        let mut f = form();
        assert!(!f.validate_all());
    }

    #[test]
    fn validate_all_passes_with_valid_data() {
        let mut f = form();
        f.name.set("Alice".to_string());
        f.email.set("alice@example.com".to_string());
        f.message
            .set("Hello, I have a question about your services.".to_string());
        assert!(f.validate_all());
    }

    #[test]
    fn validate_all_fails_with_bad_email() {
        let mut f = form();
        f.name.set("Alice".to_string());
        f.email.set("not-an-email".to_string());
        f.message.set("Hello, I have a question.".to_string());
        assert!(!f.validate_all());
        assert!(!f.email_error.read().is_empty());
    }

    #[test]
    fn validate_all_fails_with_short_message() {
        let mut f = form();
        f.name.set("Alice".to_string());
        f.email.set("alice@example.com".to_string());
        f.message.set("Hi".to_string());
        assert!(!f.validate_all());
        assert!(!f.message_error.read().is_empty());
    }

    #[test]
    fn shows_success_after_submit() {
        let mut f = form();
        f.name.set("Alice".to_string());
        f.email.set("alice@example.com".to_string());
        f.message
            .set("Hello, I have a question about your services.".to_string());
        f.validate_all();
        f.submitted.set(true);
        assert!(f.submitted.read());
    }
}
