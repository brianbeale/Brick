#[derive(Clone)]
pub struct BrickFile {
    pub name: String,
    pub size: f64,
    pub mime_type: String,
    #[cfg(brick_dom)]
    file: web_sys::File,
    #[cfg(not(brick_dom))]
    _private: (),
}

impl BrickFile {
    pub async fn read(&self) -> Result<Vec<u8>, String> {
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::{JsCast, closure::Closure};
            use wasm_bindgen_futures::JsFuture;

            let promise = js_sys::Promise::new(&mut |resolve, reject| {
                let reader = match web_sys::FileReader::new() {
                    Ok(r) => r,
                    Err(_) => {
                        let _ = reject.call1(
                            &wasm_bindgen::JsValue::NULL,
                            &wasm_bindgen::JsValue::from_str("could not create FileReader"),
                        );
                        return;
                    }
                };
                let reader_c = reader.clone();
                let onload = Closure::once(move || {
                    let result = reader_c.result().unwrap_or(wasm_bindgen::JsValue::NULL);
                    let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &result);
                });
                let onerror = Closure::once(move |_: web_sys::ProgressEvent| {
                    let _ = reject.call1(
                        &wasm_bindgen::JsValue::NULL,
                        &wasm_bindgen::JsValue::from_str("FileReader error"),
                    );
                });
                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                let _ = reader.read_as_array_buffer(&self.file);
                onload.forget();
                onerror.forget();
            });

            let val = JsFuture::from(promise)
                .await
                .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;

            let buf: js_sys::ArrayBuffer = val
                .dyn_into()
                .map_err(|_| "result was not an ArrayBuffer".to_string())?;

            Ok(js_sys::Uint8Array::new(&buf).to_vec())
        }
        #[cfg(not(brick_dom))]
        {
            Ok(Vec::new())
        }
    }
}

pub fn from_event(event: &web_sys::Event) -> Vec<BrickFile> {
    let _ = event;
    #[cfg(brick_dom)]
    {
        use wasm_bindgen::JsCast;

        let file_list = event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            .and_then(|el| el.files())
            .or_else(|| {
                event
                    .dyn_ref::<web_sys::DragEvent>()
                    .and_then(|e| e.data_transfer())
                    .and_then(|dt| dt.files())
            });

        let Some(list) = file_list else {
            return Vec::new();
        };

        (0..list.length())
            .filter_map(|i| list.get(i))
            .map(|file| BrickFile {
                name: file.name(),
                size: file.size(),
                mime_type: file.type_(),
                file,
            })
            .collect()
    }
    #[cfg(not(brick_dom))]
    {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brick_file_fields() {
        let f = BrickFile {
            name: "photo.jpg".to_string(),
            size: 1024.0,
            mime_type: "image/jpeg".to_string(),
            _private: (),
        };
        assert_eq!(f.name, "photo.jpg");
        assert_eq!(f.size, 1024.0);
        assert_eq!(f.mime_type, "image/jpeg");
    }
}
