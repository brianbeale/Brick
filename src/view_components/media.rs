use std::collections::HashMap;

pub struct ViewLeafMedia {
    tag: &'static str,
    url: String,
    attrs: HashMap<&'static str, &'static str>,
}
