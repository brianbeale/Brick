use crate::ssr::BrickError;
use crate::state_mgmt::{Load, Signal};
use serde::{Serialize, de::DeserializeOwned};

pub struct BrickSocket<T: DeserializeOwned + Serialize + Clone + 'static> {
    signal: Signal<Load<T, BrickError>>,
    #[cfg(brick_dom)]
    ws: web_sys::WebSocket,
    #[cfg(not(brick_dom))]
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize + Clone + 'static> BrickSocket<T> {
    pub fn connect(url: &str) -> Self {
        let signal: Signal<Load<T, BrickError>> = Signal::new(Load::Idle);
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::{JsCast, closure::Closure};
            let ws = web_sys::WebSocket::new(url).expect("BrickSocket: invalid WebSocket URL");
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // onmessage: deserialise and set Loaded
            let sig = signal.clone();
            let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    match serde_json::from_str::<T>(&text) {
                        Ok(msg) => sig.set(Load::Loaded(msg)),
                        Err(err) => sig.set(Load::Failed(BrickError::internal(err.to_string()))),
                    }
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>);
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            // onerror: set Failed
            let sig_err = signal.clone();
            let onerror = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                sig_err.set(Load::Failed(BrickError::internal("WebSocket error")));
            }) as Box<dyn FnMut(web_sys::Event)>);
            ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();

            return BrickSocket { signal, ws };
        }
        #[cfg(not(brick_dom))]
        {
            let _ = url;
            BrickSocket {
                signal,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    pub fn signal(&self) -> Signal<Load<T, BrickError>> {
        self.signal.clone()
    }

    pub fn send(&self, msg: &T) {
        let _ = msg;
        #[cfg(brick_dom)]
        if let Ok(json) = serde_json::to_string(msg) {
            let _ = self.ws.send_with_str(&json);
        }
    }

    pub fn close(&self) {
        #[cfg(brick_dom)]
        {
            let _ = self.ws.close();
        }
    }
}

#[cfg(brick_dom)]
impl<T: DeserializeOwned + Serialize + Clone + 'static> Drop for BrickSocket<T> {
    fn drop(&mut self) {
        let _ = self.ws.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_starts_idle() {
        let sock: BrickSocket<String> = BrickSocket::connect("ws://localhost");
        assert!(matches!(sock.signal().read(), Load::Idle));
    }

    #[test]
    fn send_and_close_do_not_panic() {
        let sock: BrickSocket<String> = BrickSocket::connect("ws://localhost");
        sock.send(&"hello".to_string());
        sock.close();
    }
}
