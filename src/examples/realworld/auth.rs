use crate::browser::BrickStorage;
use crate::state_mgmt::Signal;

pub struct AuthStore {
    pub token: Signal<String>,
    pub username: Signal<String>,
    pub image: Signal<String>,
}

impl Clone for AuthStore {
    fn clone(&self) -> Self {
        AuthStore {
            token: self.token.clone(),
            username: self.username.clone(),
            image: self.image.clone(),
        }
    }
}

thread_local! {
    static AUTH_STORE: AuthStore = {
        let token    = Signal::new(BrickStorage::get::<String>("rw_token").ok().flatten().unwrap_or_default());
        let username = Signal::new(BrickStorage::get::<String>("rw_username").ok().flatten().unwrap_or_default());
        let image    = Signal::new(BrickStorage::get::<String>("rw_image").ok().flatten().unwrap_or_default());

        token.add_observer("__persist_token", Box::new(crate::state_mgmt::observers::Effect::new(
            |v: &String| { BrickStorage::set("rw_token", v); }
        )));
        username.add_observer("__persist_username", Box::new(crate::state_mgmt::observers::Effect::new(
            |v: &String| { BrickStorage::set("rw_username", v); }
        )));
        image.add_observer("__persist_image", Box::new(crate::state_mgmt::observers::Effect::new(
            |v: &String| { BrickStorage::set("rw_image", v); }
        )));

        AuthStore { token, username, image }
    };
}

impl AuthStore {
    pub fn get() -> AuthStore {
        AUTH_STORE.with(|s| s.clone())
    }

    pub fn set_token(value: String) {
        Self::get().token.set(value);
    }
    pub fn set_username(value: String) {
        Self::get().username.set(value);
    }
    pub fn set_image(value: String) {
        Self::get().image.set(value);
    }
}

pub fn is_logged_in() -> bool {
    !AuthStore::get().token.read().is_empty()
}
