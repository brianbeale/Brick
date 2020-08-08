#[macro_export]
macro_rules! composite {
    ( $struct_name:ident, $model:expr, $kids:expr ) => {{
        // let mut model = $s.make_model();
        let children: Vec<Box<dyn ViewComponent>> = $kids;
        let (event_listeners, sharing_model) = $model.controller_methods();
        ViewComposite {
            tag: "div",
            class_name: stringify!($struct_name).to_string(),
            children,
            event_listeners,
            model: sharing_model,
        }
    }};
}

macro_rules! children {
    ( $( $child:expr ),* $(,)* ) => {
        vec![
            $( $child ),*
        ]
    };
}
