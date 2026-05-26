pub async fn read() -> Result<String, String> {
    #[cfg(brick_dom)]
    {
        use wasm_bindgen_futures::JsFuture;
        let clipboard = web_sys::window()
            .ok_or_else(|| "no window".to_string())?
            .navigator()
            .clipboard();
        JsFuture::from(clipboard.read_text())
            .await
            .map_err(|e| format!("{:?}", e))?
            .as_string()
            .ok_or_else(|| "clipboard value was not a string".to_string())
    }
    #[cfg(not(brick_dom))]
    {
        Ok(String::new())
    }
}

pub async fn write(text: &str) -> Result<(), String> {
    let _ = text;
    #[cfg(brick_dom)]
    {
        use wasm_bindgen_futures::JsFuture;
        let clipboard = web_sys::window()
            .ok_or_else(|| "no window".to_string())?
            .navigator()
            .clipboard();
        JsFuture::from(clipboard.write_text(text))
            .await
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }
    #[cfg(not(brick_dom))]
    {
        Ok(())
    }
}
