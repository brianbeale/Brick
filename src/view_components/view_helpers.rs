#[macro_export]
macro_rules! make_composite {
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
