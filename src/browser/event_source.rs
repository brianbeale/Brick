use crate::ssr::BrickError;
use crate::state_mgmt::{Load, Signal};
use serde::de::DeserializeOwned;

pub struct BrickEventSource<T: DeserializeOwned + Clone + 'static> {
    signal: Signal<Load<T, BrickError>>,
    #[cfg(brick_dom)]
    es: web_sys::EventSource,
    #[cfg(not(brick_dom))]
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned + Clone + 'static> BrickEventSource<T> {
    pub fn connect(url: &str) -> Self {
        let signal: Signal<Load<T, BrickError>> = Signal::new(Load::Idle);
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::{JsCast, closure::Closure};
            let es = web_sys::EventSource::new(url).expect("BrickEventSource: invalid URL");

            // onmessage: deserialise and set Loaded
            let sig = signal.clone();
            let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    match serde_json::from_str::<T>(&text) {
                        Ok(evt) => sig.set(Load::Loaded(evt)),
                        Err(err) => sig.set(Load::Failed(BrickError::internal(err.to_string()))),
                    }
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>);
            es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            // onerror: set Failed
            let sig_err = signal.clone();
            let onerror = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                sig_err.set(Load::Failed(BrickError::internal("EventSource error")));
            }) as Box<dyn FnMut(web_sys::Event)>);
            es.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();

            return BrickEventSource { signal, es };
        }
        #[cfg(not(brick_dom))]
        {
            let _ = url;
            BrickEventSource {
                signal,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    pub fn signal(&self) -> Signal<Load<T, BrickError>> {
        self.signal.clone()
    }

    pub fn close(&self) {
        #[cfg(brick_dom)]
        self.es.close();
    }
}

#[cfg(brick_dom)]
impl<T: DeserializeOwned + Clone + 'static> Drop for BrickEventSource<T> {
    fn drop(&mut self) {
        self.es.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_starts_idle() {
        let sse: BrickEventSource<String> = BrickEventSource::connect("https://example.com/events");
        assert!(matches!(sse.signal().read(), Load::Idle));
    }

    #[test]
    fn close_does_not_panic() {
        let sse: BrickEventSource<String> = BrickEventSource::connect("https://example.com/events");
        sse.close();
    }
}
