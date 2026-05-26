#[derive(Debug, Clone)]
pub struct Coords {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

pub async fn get() -> Result<Coords, String> {
    #[cfg(brick_dom)]
    {
        use wasm_bindgen::{JsCast, closure::Closure};
        use wasm_bindgen_futures::JsFuture;

        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let geo = match web_sys::window().and_then(|w| w.navigator().geolocation().ok()) {
                Some(g) => g,
                None => {
                    let _ = reject.call1(
                        &wasm_bindgen::JsValue::NULL,
                        &wasm_bindgen::JsValue::from_str("Geolocation API unavailable"),
                    );
                    return;
                }
            };

            let success = Closure::once(move |pos: web_sys::Position| {
                let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &pos);
            });
            let error = Closure::once(move |err: web_sys::PositionError| {
                let msg = err.message();
                let _ = reject.call1(
                    &wasm_bindgen::JsValue::NULL,
                    &wasm_bindgen::JsValue::from_str(&msg),
                );
            });
            geo.get_current_position_with_error_callback(
                success.as_ref().unchecked_ref(),
                Some(error.as_ref().unchecked_ref()),
            )
            .unwrap_or(());
            success.forget();
            error.forget();
        });

        let val = JsFuture::from(promise)
            .await
            .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;

        let pos: web_sys::Position = val
            .dyn_into()
            .map_err(|_| "unexpected geolocation response".to_string())?;
        let c = pos.coords();

        Ok(Coords {
            latitude: c.latitude(),
            longitude: c.longitude(),
            accuracy: c.accuracy(),
            altitude: c.altitude(),
            altitude_accuracy: c.altitude_accuracy(),
            heading: c.heading(),
            speed: c.speed(),
        })
    }
    #[cfg(not(brick_dom))]
    {
        Ok(Coords {
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 0.0,
            altitude: None,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coords_fields_accessible() {
        let c = Coords {
            latitude: 51.5,
            longitude: -0.1,
            accuracy: 10.0,
            altitude: None,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        };
        assert_eq!(c.latitude, 51.5);
        assert_eq!(c.longitude, -0.1);
    }
}
