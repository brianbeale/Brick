use crate::ssr::BrickError;
use serde::{Serialize, de::DeserializeOwned};

pub struct BrickStorage;

impl BrickStorage {
    /// Retrieve and deserialise a value from `localStorage`.
    ///
    /// Returns:
    /// - `Ok(Some(v))` — key exists and deserialised successfully
    /// - `Ok(None)` — key does not exist in storage
    /// - `Err(BrickError)` — deserialisation failed (key exists but value is malformed)
    pub fn get<T: DeserializeOwned>(key: &str) -> Result<Option<T>, BrickError> {
        let _ = key;
        #[cfg(brick_dom)]
        {
            let storage = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .and_then(|s| s);
            let storage = match storage {
                Some(s) => s,
                None => return Ok(None),
            };
            let raw = match storage.get_item(key).ok().flatten() {
                Some(r) => r,
                None => return Ok(None),
            };
            return serde_json::from_str(&raw).map(Some).map_err(|e| {
                BrickError::internal(format!(
                    "storage deserialisation error for key '{}': {}",
                    key, e
                ))
            });
        }
        #[cfg(not(brick_dom))]
        Ok(None)
    }

    pub fn set<T: Serialize>(key: &str, value: &T) {
        let _ = (key, value);
        #[cfg(brick_dom)]
        if let (Ok(json), Some(Ok(Some(storage)))) = (
            serde_json::to_string(value),
            web_sys::window().map(|w| w.local_storage()),
        ) {
            let _ = storage.set_item(key, &json);
        }
    }

    pub fn remove(key: &str) {
        let _ = key;
        #[cfg(brick_dom)]
        if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = storage.remove_item(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_ok_none_in_test_mode() {
        let v: Result<Option<String>, BrickError> = BrickStorage::get("any");
        assert!(v.is_ok());
        assert!(v.unwrap().is_none());
    }

    #[test]
    fn set_and_remove_do_not_panic() {
        BrickStorage::set("k", &"v");
        BrickStorage::remove("k");
    }
}
