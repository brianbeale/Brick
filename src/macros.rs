#![allow(unused)]

#[macro_export]
macro_rules! console_log {
    ( $text:expr ) => {
        web_sys::console::log_1(&wasm_bindgen::prelude::JsValue::from_str(&$text));
    };
}

macro_rules! cb {
    ( $body:expr ) => {
        // use wasm_bindgen::JsCast;
        Closure::wrap(Box::new($body) as Box<dyn FnMut(web_sys::Event)>)
        // can't use here because it creates a temporary value
        // .as_ref().unchecked_ref()
    };
}

// macro_rules! wrap_text {
//     ( $text:expr ) => {
//         Node::Text($text)
//     };
// }

// macro_rules! div {
//     ( $( $child:item ),* ) => {
//         Node::Element {
//             tag: "div",
//             children: vec![ $( $child ),* ],
//         }
//     }
// }

// expect one or many args, each of which could be str or Node (expr vs item)
// macro_rules! h1 {
//   ( $( $child:expr ),* ) => {
//     Node::Element {
//       tag: "h1",
//       children: {
//         $(  )*.collect()
//       }
//     }
//   }
// }

// macro_rules! handle_click {
//   ( $element:ident, $handler:expr ) => {
//     use wasm_bindgen::JsCast;
//     let callback = cb!{ $handler };
//     $element.add_event_listener_with_callback("click",
//       callback.as_ref().unchecked_ref() )?;
//     // TODO: fix memory leak
//     // Rust needs to forget this callback so it doesn't destroy it
//     callback.forget();
//   }
