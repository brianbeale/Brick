use crate::view_components::*;

#[model]
pub struct Card {
    #[prop]
    pub title: String,
    #[slot]
    pub children: Slot,
}

#[view(Card)]
fn render() -> Box<ViewComposite> {
    children! {
        h2(&my.title),
        my.children,
    }
}
