use super::super::api::{self, UpdateUserRequest};
use super::super::auth::AuthStore;
use super::super::routes::{RwRoute, navigate};
use crate::browser::BrickStorage;
use crate::view_components::*;

#[model]
pub struct SettingsPage {
    #[default(String::new())]
    pub image: String,
    #[default(String::new())]
    pub username: String,
    #[default(String::new())]
    pub bio: String,
    #[default(String::new())]
    pub email: String,
    #[default(String::new())]
    pub password: String,
    #[default(String::new())]
    pub error: String,
    #[default(false)]
    pub has_error: bool,
    #[default(false)]
    pub saving: bool,
}

#[controller]
impl SettingsPage {
    #[on(mount)]
    pub fn init(&mut self) {
        let auth = AuthStore::get();
        self.image.set(auth.image.read());
        self.username.set(auth.username.read());
        let token = auth.token.read();
        let image = self.image.clone();
        let username = self.username.clone();
        let bio = self.bio.clone();
        let email = self.email.clone();
        #[cfg(brick_dom)]
        spawn_local(async move {
            if let Ok(user) = api::get_current_user(&token).await {
                image.set(user.image.unwrap_or_default());
                username.set(user.username);
                bio.set(user.bio.unwrap_or_default());
                email.set(user.email);
            }
        });
    }

    pub fn save(&mut self) {
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            return;
        }
        let image_val = self.image.read();
        let username_val = self.username.read();
        let bio_val = self.bio.read();
        let email_val = self.email.read();
        let password_val = self.password.read();
        let error = self.error.clone();
        let has_error = self.has_error.clone();
        let saving = self.saving.clone();
        saving.set(true);
        #[cfg(brick_dom)]
        spawn_local(async move {
            let req = UpdateUserRequest {
                email: if email_val.is_empty() {
                    None
                } else {
                    Some(email_val)
                },
                username: if username_val.is_empty() {
                    None
                } else {
                    Some(username_val.clone())
                },
                bio: if bio_val.is_empty() {
                    None
                } else {
                    Some(bio_val)
                },
                image: if image_val.is_empty() {
                    None
                } else {
                    Some(image_val.clone())
                },
            };
            saving.set(false);
            match api::update_user(&token, req).await {
                Ok(user) => {
                    let auth = AuthStore::get();
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

    pub fn logout(&mut self) {
        let auth = AuthStore::get();
        auth.token.set(String::new());
        auth.username.set(String::new());
        auth.image.set(String::new());
        BrickStorage::remove("rw_token");
        BrickStorage::remove("rw_username");
        BrickStorage::remove("rw_image");
        #[cfg(brick_dom)]
        navigate(RwRoute::Home);
    }
}

#[view(SettingsPage)]
fn render() -> Box<ViewComposite> {
    style! {
        .settings-page { max-width: 720px; margin: 2rem auto; padding: 0 1rem; }
        .settings-page h1 { font-size: 1.75rem; font-weight: 700; margin-bottom: 1.5rem; text-align: center; }
        .settings-form { display: flex; flex-direction: column; gap: 1rem; }
        .error-list { background: color-mix(in srgb, #e53e3e 10%, transparent); border: 1px solid #e53e3e; border-radius: 4px; padding: 0.75rem 1rem; color: #e53e3e; }
        .logout-btn { margin-top: 1rem; padding: 0.5rem 1rem; background: none; border: 1px solid #e53e3e; color: #e53e3e; border-radius: 3px; cursor: pointer; }
        .logout-btn:hover { background: #e53e3e; color: white; }
    }
    children! {
        div {
            class("settings-page"),
            h1("Your Settings"),
            when!(my.has_error,
                p(live!("{my.error}")).c("error-list"),
            ),
            div {
                class("settings-form"),
                input().attr("type", "url").attr("placeholder", "URL of profile picture").bind(&my.image),
                input().attr("type", "text").attr("placeholder", "Username").bind(&my.username),
                input().attr("type", "text").attr("placeholder", "Short bio about you").bind(&my.bio),
                input().attr("type", "email").attr("placeholder", "Email").bind(&my.email),
                input().attr("type", "password").attr("placeholder", "New Password").bind(&my.password),
                button("Update Settings").primary().trigger(&my.save),
            },
            hr(),
            button("Or click here to logout.").c("logout-btn").trigger(&my.logout),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn page() -> SettingsPage {
        SettingsPage { ..cascade() }
    }

    #[test]
    fn renders_settings_heading() {
        assert!(
            crate::view_components::render_to_html(&*page().into_component()).contains("Your Settings")
        );
    }

    #[test]
    fn logout_clears_auth() {
        let mut p = page();
        let auth = AuthStore::get();
        auth.token.set("some-token".to_string());
        auth.username.set("alice".to_string());
        p.logout();
        assert!(auth.token.read().is_empty());
        assert!(auth.username.read().is_empty());
    }

    #[test]
    fn initial_saving_false() {
        assert!(!page().saving.read());
    }
}
