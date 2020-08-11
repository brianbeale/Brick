#[macro_export]
macro_rules! composite {
    ( $struct_name:ident, $event_listeners:expr, $sharing_model:expr, $kids:expr ) => {{
        // let mut model = $s.make_model();
        let children: Vec<Box<dyn ViewComponent>> = $kids;
        Box::new(ViewComposite {
            tag: "div",
            class_name: stringify!($struct_name).to_string(),
            children,
            event_listeners: $event_listeners,
            model: $sharing_model,
        })
    }};
}

macro_rules! children {
    ( $( $child:expr ),* $(,)* ) => {{
        vec![
            $( $child ),*
        ]
    }};
}

// macro_rules! composite_discriminator {
//     // ( $name:ty {$( $tail:tt )*} ) => {
//     //     box_and_render!($name {$( $tail )*})
//     // };
//     ( $( $name:ident ($inner:expr) $(.)*) ) => {

//     }
//     ( $( $anything_else:tt )* ) => {
//         $( $anything_else )*
//     }
// }

// macro_rules! box_and_render {
//     ( $composite_struct:expr ) => {
//         Box::new($composite_struct.render())
//     };
// }
