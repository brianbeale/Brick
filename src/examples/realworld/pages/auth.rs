use super::super::api;
use super::super::auth::AuthStore;
use super::super::routes::{RwRoute, navigate};
use crate::view_components::*;

// ── Login ─────────────────────────────────────────────────────────────────────

#[model]
pub struct LoginPage {
    #[validate(required, email)]
    pub email: String,
    #[validate(required)]
    pub password: String,
    #[default(String::new())]
    pub error: String,
    #[default(false)]
    pub has_error: bool,
}

#[controller]
impl LoginPage {
    #[on(submit)]
    pub fn login(&mut self) {
        let email = self.email.read();
        let password = self.password.read();
        let error = self.error.clone();
        let has_error = self.has_error.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            match api::login(&email, &password).await {
                Ok(user) => {
                    let auth = AuthStore::get();
                    auth.token.set(user.token);
                    auth.username.set(user.username);
                    auth.image.set(user.image.unwrap_or_default());
                    navigate(RwRoute::Home);
                }
                Err(e) => {
                    error.set(e);
                    has_error.set(true);
                }
            }
        });
    }
}

#[view(LoginPage)]
fn render() -> Box<ViewComposite> {
    style! {
        .auth-page { max-width: 540px; margin: 2rem auto; padding: 0 1rem; text-align: center; }
        .auth-page h1 { font-size: 1.75rem; font-weight: 700; margin-bottom: 0.25rem; }
        .auth-page .sub { color: #5cb85c; margin-bottom: 1.5rem; }
        .auth-form { display: flex; flex-direction: column; gap: 1rem; text-align: left; }
        .error-list { background: color-mix(in srgb, #e53e3e 10%, transparent); border: 1px solid #e53e3e; border-radius: 4px; padding: 0.75rem 1rem; color: #e53e3e; margin-bottom: 0.5rem; }
        .field-error { font-size: 0.8rem; color: #e53e3e; min-height: 1.1rem; }
    }
    children! {
        div {
            class("auth-page"),
            h1("Sign in"),
            a("Need an account?").c("sub").attr("href", "/register"),
            div {
                class("auth-form"),
                when!(my.has_error,
                    p(live!("{my.error}")).c("error-list"),
                ),
                div {
                    class(""),
                    input().attr("type", "email").attr("placeholder", "Email").bind(&my.email),
                    when!(my.email_touched,
                        p(live!("{my.email_error}")).c("field-error"),
                    ),
                },
                div {
                    class(""),
                    input().attr("type", "password").attr("placeholder", "Password").bind(&my.password),
                    when!(my.password_touched,
                        p(live!("{my.password_error}")).c("field-error"),
                    ),
                },
                when!(my.valid,
                    button("Sign in").primary().submit(&my.login),
                    button("Sign in").primary().attr("disabled", "true"),
                ),
            },
        },
    }
}

// ── Register ──────────────────────────────────────────────────────────────────

#[model]
pub struct RegisterPage {
    #[validate(required, min_length(1, "Username is required"))]
    pub username: String,
    #[validate(required, email)]
    pub email: String,
    #[validate(required, min_length(8, "Password must be at least 8 characters"))]
    pub password: String,
    #[default(String::new())]
    pub error: String,
    #[default(false)]
    pub has_error: bool,
}

#[controller]
impl RegisterPage {
    #[on(submit)]
    pub fn register(&mut self) {
        let username = self.username.read();
        let email = self.email.read();
        let password = self.password.read();
        let error = self.error.clone();
        let has_error = self.has_error.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            match api::register(&username, &email, &password).await {
                Ok(user) => {
                    let auth = AuthStore::get();
                    auth.token.set(user.token);
                    auth.username.set(user.username);
                    auth.image.set(user.image.unwrap_or_default());
                    navigate(RwRoute::Home);
                }
                Err(e) => {
                    error.set(e);
                    has_error.set(true);
                }
            }
        });
    }
}

#[view(RegisterPage)]
fn render() -> Box<ViewComposite> {
    style! {
        .auth-page { max-width: 540px; margin: 2rem auto; padding: 0 1rem; text-align: center; }
        .auth-page h1 { font-size: 1.75rem; font-weight: 700; margin-bottom: 0.25rem; }
        .auth-page .sub { color: #5cb85c; margin-bottom: 1.5rem; }
        .auth-form { display: flex; flex-direction: column; gap: 1rem; text-align: left; }
        .error-list { background: color-mix(in srgb, #e53e3e 10%, transparent); border: 1px solid #e53e3e; border-radius: 4px; padding: 0.75rem 1rem; color: #e53e3e; margin-bottom: 0.5rem; }
        .field-error { font-size: 0.8rem; color: #e53e3e; min-height: 1.1rem; }
    }
    children! {
        div {
            class("auth-page"),
            h1("Sign up"),
            a("Have an account?").c("sub").attr("href", "/login"),
            div {
                class("auth-form"),
                when!(my.has_error,
                    p(live!("{my.error}")).c("error-list"),
                ),
                div {
                    class(""),
                    input().attr("type", "text").attr("placeholder", "Username").bind(&my.username),
                    when!(my.username_touched,
                        p(live!("{my.username_error}")).c("field-error"),
                    ),
                },
                div {
                    class(""),
                    input().attr("type", "email").attr("placeholder", "Email").bind(&my.email),
                    when!(my.email_touched,
                        p(live!("{my.email_error}")).c("field-error"),
                    ),
                },
                div {
                    class(""),
                    input().attr("type", "password").attr("placeholder", "Password").bind(&my.password),
                    when!(my.password_touched,
                        p(live!("{my.password_error}")).c("field-error"),
                    ),
                },
                when!(my.valid,
                    button("Sign up").primary().submit(&my.register),
                    button("Sign up").primary().attr("disabled", "true"),
                ),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    #[test]
    fn login_renders() {
        let p = LoginPage { ..cascade() };
        assert!(crate::view_components::render_to_html(&*p.into_component()).contains("Sign in"));
    }

    #[test]
    fn register_renders() {
        let p = RegisterPage { ..cascade() };
        assert!(crate::view_components::render_to_html(&*p.into_component()).contains("Sign up"));
    }

    #[test]
    fn login_initial_not_valid() {
        let p = LoginPage { ..cascade() };
        assert!(!p.valid.read());
    }

    #[test]
    fn register_initial_not_valid() {
        let p = RegisterPage { ..cascade() };
        assert!(!p.valid.read());
    }

    #[test]
    fn login_valid_with_good_data() {
        let mut p = LoginPage { ..cascade() };
        p.email.set("user@example.com".to_string());
        p.password.set("password".to_string());
        assert!(p.validate_all());
    }

    #[test]
    fn register_valid_with_good_data() {
        let mut p = RegisterPage { ..cascade() };
        p.username.set("alice".to_string());
        p.email.set("alice@example.com".to_string());
        p.password.set("secretpassword".to_string());
        assert!(p.validate_all());
    }

    #[test]
    fn login_invalid_email_fails() {
        let mut p = LoginPage { ..cascade() };
        p.email.set("notanemail".to_string());
        p.password.set("password".to_string());
        assert!(!p.validate_all());
    }
}
