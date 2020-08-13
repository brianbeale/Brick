#![allow(unused)]
use super::super::ViewComponent;

pub struct ViewLeafBlank {}
impl ViewComponent for ViewLeafBlank {
    fn html(&self) -> String {
        String::new()
    }
    fn attach_listeners(&self) {}
}
pub fn blank() -> Box<ViewLeafBlank> {
    Box::new(ViewLeafBlank {})
}
