use crate::ssr::BrickError;
use serde::{Serialize, de::DeserializeOwned};

pub struct FetchBuilder {
    pub(crate) url: String,
    pub(crate) headers: Vec<(String, String)>,
}

pub fn fetch(url: &str) -> FetchBuilder {
    FetchBuilder {
        url: url.to_string(),
        headers: Vec::new(),
    }
}

/// POST `body` bytes to `url` and return the response body as raw bytes.
/// Used by the `#[server]` macro's client-side stub.
#[cfg(brick_dom)]
pub async fn fetch_bytes(url: &str, body: Vec<u8>) -> Result<Vec<u8>, BrickError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    let body_js = js_sys::Uint8Array::from(body.as_slice());
    opts.body(Some(&body_js));

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| BrickError::internal(format!("{:?}", e)))?;
    request
        .headers()
        .set("Content-Type", "application/octet-stream")
        .map_err(|e| BrickError::internal(format!("{:?}", e)))?;

    let window = web_sys::window().ok_or_else(|| BrickError::internal("no window"))?;
    let resp_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| BrickError::internal(format!("{:?}", e)))?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| BrickError::internal("response was not a Response"))?;

    if !resp.ok() {
        return Err(BrickError::http(
            resp.status() as u32,
            format!("HTTP {}", resp.status()),
        ));
    }

    let array_buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| BrickError::internal(format!("{:?}", e)))?,
    )
    .await
    .map_err(|e| BrickError::internal(format!("{:?}", e)))?;

    Ok(js_sys::Uint8Array::new(&array_buffer).to_vec())
}

/// Stub so code that references `fetch_bytes` compiles on native targets.
#[cfg(not(brick_dom))]
pub async fn fetch_bytes(_url: &str, _body: Vec<u8>) -> Result<Vec<u8>, BrickError> {
    Err(BrickError::internal(
        "fetch_bytes is only available in browser builds",
    ))
}

impl FetchBuilder {
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub async fn get<T: DeserializeOwned>(self) -> Result<T, BrickError> {
        self.request("GET", None).await
    }

    pub async fn delete<T: DeserializeOwned>(self) -> Result<T, BrickError> {
        self.request("DELETE", None).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(self, body: &B) -> Result<T, BrickError> {
        let json = serde_json::to_string(body).map_err(|e| BrickError::internal(e.to_string()))?;
        self.request("POST", Some(json)).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(self, body: &B) -> Result<T, BrickError> {
        let json = serde_json::to_string(body).map_err(|e| BrickError::internal(e.to_string()))?;
        self.request("PUT", Some(json)).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(self, body: &B) -> Result<T, BrickError> {
        let json = serde_json::to_string(body).map_err(|e| BrickError::internal(e.to_string()))?;
        self.request("PATCH", Some(json)).await
    }

    #[cfg(brick_dom)]
    async fn request<T: DeserializeOwned>(
        self,
        method: &str,
        body: Option<String>,
    ) -> Result<T, BrickError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let mut opts = web_sys::RequestInit::new();
        opts.method(method);
        if let Some(ref b) = body {
            opts.body(Some(&wasm_bindgen::JsValue::from_str(b)));
        }

        let request = web_sys::Request::new_with_str_and_init(&self.url, &opts)
            .map_err(|e| BrickError::internal(format!("{:?}", e)))?;

        let headers = request.headers();
        if body.is_some() {
            headers
                .set("Content-Type", "application/json")
                .map_err(|e| BrickError::internal(format!("{:?}", e)))?;
        }
        for (k, v) in &self.headers {
            headers
                .set(k, v)
                .map_err(|e| BrickError::internal(format!("{:?}", e)))?;
        }

        let window = web_sys::window().ok_or_else(|| BrickError::internal("no window"))?;
        let resp_val = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| BrickError::internal(format!("{:?}", e)))?;

        let resp: web_sys::Response = resp_val
            .dyn_into()
            .map_err(|_| BrickError::internal("response was not a Response"))?;

        if !resp.ok() {
            return Err(BrickError::http(
                resp.status() as u32,
                format!("HTTP {}", resp.status()),
            ));
        }

        let text = JsFuture::from(
            resp.text()
                .map_err(|e| BrickError::internal(format!("{:?}", e)))?,
        )
        .await
        .map_err(|e| BrickError::internal(format!("{:?}", e)))?
        .as_string()
        .ok_or_else(|| BrickError::internal("response body was not a string"))?;

        serde_json::from_str(&text).map_err(|e| BrickError::internal(e.to_string()))
    }

    #[cfg(not(brick_dom))]
    async fn request<T: DeserializeOwned>(
        self,
        _method: &str,
        _body: Option<String>,
    ) -> Result<T, BrickError> {
        Err(BrickError::internal("fetch not available in test mode"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_stores_url() {
        let b = fetch("https://api.example.com/todos");
        assert_eq!(b.url, "https://api.example.com/todos");
    }

    #[test]
    fn header_accumulates() {
        let b = fetch("https://example.com")
            .header("Authorization", "Bearer token")
            .header("Accept", "application/json");
        assert_eq!(b.headers.len(), 2);
        assert_eq!(
            b.headers[0],
            ("Authorization".into(), "Bearer token".into())
        );
        assert_eq!(b.headers[1], ("Accept".into(), "application/json".into()));
    }
}
