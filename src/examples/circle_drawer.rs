use crate::view_components::svg::shapes::svg_circle;
use crate::view_components::svg::*;
use crate::view_components::*;

#[derive(Clone, Default, PartialEq)]
pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub selected: bool,
}

impl Circle {
    fn contains(&self, x: f64, y: f64) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt() <= self.r
    }
}

#[model]
pub struct CircleDrawer {
    #[default(vec![])]
    pub circles: Vec<Circle>,
    #[default(vec![])]
    pub undo_stack: Vec<Vec<Circle>>,
    #[default(vec![])]
    pub redo_stack: Vec<Vec<Circle>>,
    #[default(vec![])]
    pub pre_dialog_snapshot: Vec<Circle>,
    #[default(false)]
    pub dialog_open: bool,
}

// Methods not wired as DOM actions live in a plain impl block.
impl CircleDrawer {
    pub fn handle_click(&mut self, x: f64, y: f64) {
        let circles = self.circles.read();

        let nearest_idx = circles
            .iter()
            .enumerate()
            .filter(|(_, c)| c.contains(x, y))
            .min_by(|(_, a), (_, b)| {
                let da = (a.x - x).hypot(a.y - y);
                let db = (b.x - x).hypot(b.y - y);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i);

        if let Some(idx) = nearest_idx {
            self.pre_dialog_snapshot.set(circles.clone());
            let mut updated = circles;
            for (i, c) in updated.iter_mut().enumerate() {
                c.selected = i == idx;
            }
            self.circles.set(updated);
            self.dialog_open.set(true);
        } else {
            let mut undo = self.undo_stack.read();
            undo.push(circles.clone());
            self.undo_stack.set(undo);
            self.redo_stack.set(vec![]);

            let mut updated = circles;
            updated.iter_mut().for_each(|c| c.selected = false);
            updated.push(Circle {
                x,
                y,
                r: 30.0,
                selected: false,
            });
            self.circles.set(updated);
        }
    }
}

#[controller]
impl CircleDrawer {
    #[on(click, coords)]
    pub fn canvas_click(&mut self, client_x: f64, client_y: f64) {
        #[cfg(brick_dom)]
        {
            if let Some(svg) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|doc| doc.query_selector("svg.circle-canvas").ok().flatten())
            {
                let rect = svg.get_bounding_client_rect();
                self.handle_click(client_x - rect.left(), client_y - rect.top());
            }
        }
    }

    #[on(input, f64)]
    pub fn adjust_radius(&mut self, v: f64) {
        let mut circles = self.circles.read();
        if let Some(c) = circles.iter_mut().find(|c| c.selected) {
            c.r = v.max(5.0);
        }
        self.circles.set(circles);
    }

    pub fn undo(&mut self) {
        let mut undo = self.undo_stack.read();
        if let Some(prev) = undo.pop() {
            let mut redo = self.redo_stack.read();
            redo.push(self.circles.read());
            self.redo_stack.set(redo);
            self.circles.set(prev);
            self.undo_stack.set(undo);
        }
    }

    pub fn redo(&mut self) {
        let mut redo = self.redo_stack.read();
        if let Some(next) = redo.pop() {
            let mut undo = self.undo_stack.read();
            undo.push(self.circles.read());
            self.undo_stack.set(undo);
            self.circles.set(next);
            self.redo_stack.set(redo);
        }
    }

    pub fn close_dialog(&mut self) {
        let snapshot = self.pre_dialog_snapshot.read();
        let mut circles = self.circles.read();
        circles.iter_mut().for_each(|c| c.selected = false);
        if circles != snapshot {
            let mut undo = self.undo_stack.read();
            undo.push(snapshot);
            self.undo_stack.set(undo);
            self.redo_stack.set(vec![]);
        }
        self.circles.set(circles);
        self.dialog_open.set(false);
    }
}

fn selected_radius(circles: &[Circle]) -> f64 {
    circles
        .iter()
        .find(|c| c.selected)
        .map(|c| c.r)
        .unwrap_or(30.0)
}

#[view(CircleDrawer)]
fn render() -> Box<ViewComposite> {
    style! {
        .circle-drawer { display: flex; flex-direction: column; gap: 0.75rem; max-width: 420px; }
        .circle-controls { display: flex; gap: 0.5rem; }
        .circle-controls button { flex: 1; }
        .circle-canvas { border: 1px solid var(--brick-border); border-radius: 0.375rem; display: block; cursor: crosshair; }
        .circle-dialog { border: 1px solid var(--brick-border); border-radius: 0.5rem; padding: 1rem; background: var(--brick-bg); display: flex; flex-direction: column; gap: 0.75rem; }
        .circle-dialog h3 { margin: 0; font-size: 1rem; }
        .circle-dialog-row { display: flex; align-items: center; gap: 0.75rem; font-size: 0.85rem; }
        .circle-dialog-row input[type=range] { flex: 1; }
    }
    children! {
        h2("Circle Drawer"),
        div {
            class("circle-drawer"),
            div {
                class("circle-controls"),
                button("Undo").secondary().trigger(&my.undo),
                button("Redo").secondary().trigger(&my.redo),
            },
            SvgViewMapped::new(my.circles.clone(), |circles: Vec<Circle>| {
                let mut g = svg_group();
                for c in &circles {
                    let fill = if c.selected { "#4a90d9" } else { "#d0d0d0" };
                    g = g.child(
                        svg_circle(c.x, c.y, c.r)
                            .fill(fill)
                            .stroke("#333")
                            .width(1.0)
                            .attr("cursor", "pointer")
                    );
                }
                svg_root("400", "400")
                    .view_box(0.0, 0.0, 400.0, 400.0)
                    .attr("class", "circle-canvas")
                    .attr("data-brick-action", "canvas_click")
                    .child(g)
                    .into_component()
            }),
            Box::new(ViewConditional::new(
                my.dialog_open.clone(),
                div {
                    class("circle-dialog"),
                    h3("Adjust circle radius"),
                    div {
                        class("circle-dialog-row"),
                        label("Radius:"),
                        input()
                            .attr("type", "range")
                            .attr("min", "5")
                            .attr("max", "100")
                            .attr("step", "1")
                            .attr("value", &selected_radius(&my.circles.read()).to_string())
                            .trigger(&my.adjust_radius),
                    },
                    button("Close").secondary().trigger(&my.close_dialog),
                },
                None,
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn make() -> CircleDrawer {
        CircleDrawer { ..cascade() }
    }

    #[test]
    fn renders_heading() {
        assert!(
            crate::view_components::render_to_html(&*make().into_component()).contains("Circle Drawer")
        );
    }

    #[test]
    fn starts_with_no_circles() {
        assert!(make().circles.read().is_empty());
    }

    #[test]
    fn click_empty_adds_circle() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        assert_eq!(cd.circles.read().len(), 1);
        let circles = cd.circles.read();
        let c = &circles[0];
        assert_eq!(c.x, 100.0);
        assert_eq!(c.y, 100.0);
        assert_eq!(c.r, 30.0);
    }

    #[test]
    fn add_pushes_undo_state() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        assert_eq!(cd.undo_stack.read().len(), 1);
        assert!(cd.undo_stack.read()[0].is_empty());
    }

    #[test]
    fn click_on_circle_opens_dialog() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.handle_click(100.0, 100.0);
        assert!(cd.dialog_open.read());
        assert!(cd.circles.read()[0].selected);
    }

    #[test]
    fn adjust_radius_updates_selected() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.handle_click(100.0, 100.0);
        cd.adjust_radius(50.0);
        assert_eq!(cd.circles.read()[0].r, 50.0);
    }

    #[test]
    fn adjust_radius_clamps_minimum() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.handle_click(100.0, 100.0);
        cd.adjust_radius(1.0);
        assert_eq!(cd.circles.read()[0].r, 5.0);
    }

    #[test]
    fn close_dialog_pushes_undo_when_radius_changed() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        let undo_len_before_select = cd.undo_stack.read().len();
        cd.handle_click(100.0, 100.0);
        cd.adjust_radius(50.0);
        cd.close_dialog();
        assert!(!cd.dialog_open.read());
        assert_eq!(cd.undo_stack.read().len(), undo_len_before_select + 1);
    }

    #[test]
    fn close_dialog_no_undo_when_radius_unchanged() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        let undo_before = cd.undo_stack.read().len();
        cd.handle_click(100.0, 100.0);
        cd.close_dialog();
        assert_eq!(cd.undo_stack.read().len(), undo_before);
    }

    #[test]
    fn undo_removes_last_added_circle() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.handle_click(200.0, 200.0);
        assert_eq!(cd.circles.read().len(), 2);
        cd.undo();
        assert_eq!(cd.circles.read().len(), 1);
    }

    #[test]
    fn redo_restores_undone_circle() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.undo();
        assert!(cd.circles.read().is_empty());
        cd.redo();
        assert_eq!(cd.circles.read().len(), 1);
    }

    #[test]
    fn new_action_clears_redo_stack() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.undo();
        cd.handle_click(200.0, 200.0);
        assert!(cd.redo_stack.read().is_empty());
    }

    #[test]
    fn click_outside_all_circles_does_not_select() {
        let mut cd = make();
        cd.handle_click(100.0, 100.0);
        cd.handle_click(300.0, 300.0);
        assert!(!cd.dialog_open.read());
        assert_eq!(cd.circles.read().len(), 2);
    }

    #[test]
    fn select_nearest_when_circles_overlap() {
        let mut cd = make();
        // circle 0 at (100,100); circle 1 at (135,100) — 35px apart, both r=30 so they overlap
        cd.handle_click(100.0, 100.0);
        cd.handle_click(135.0, 100.0); // 35 > 30, outside circle 0, creates new circle
        assert_eq!(cd.circles.read().len(), 2);
        let undo_len = cd.undo_stack.read().len();
        // click at (119,100): da=19 to circle 0, db=16 to circle 1 → circle 1 is nearer
        cd.handle_click(119.0, 100.0);
        assert!(cd.dialog_open.read());
        let circles = cd.circles.read();
        assert!(!circles[0].selected);
        assert!(
            circles[1].selected,
            "circle 1 should be selected (closer center)"
        );
        assert_eq!(cd.undo_stack.read().len(), undo_len);
    }
}
